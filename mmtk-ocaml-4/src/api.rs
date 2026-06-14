//! C-exported API for the OCaml 4.14 MMTk binding.
//!
//! The patched OCaml 4.14 runtime links against libmmtk_ocaml_4.a and calls
//! these functions in place of its own allocator and GC driver.
//!
//! Function naming convention: mmtk_ocaml4_*
//!
//! Bring-up sequence:
//!   Phase 1 (NoGC):    mmtk_init, mmtk_bind_mutator, mmtk_alloc, mmtk_post_alloc
//!   Phase 2 (MarkSweep): + STW, scanning, destroy_mutator
//!   Phase 3 (Immix):   + copy, write barrier
//!
//! TODO (agent): implement each function. Start with mmtk_init and mmtk_alloc
//! for the NoGC phase; add the rest incrementally.

use std::ffi::CStr;

use mmtk::memory_manager;
use mmtk::util::opaque_pointer::{OpaquePointer, VMMutatorThread, VMThread};
use mmtk::util::{Address, ObjectReference};
use mmtk::AllocationSemantics;
use mmtk::MMTKBuilder;

use mmtk_ocaml_common::header::{make_header, WORD_SIZE};
use mmtk_ocaml_common::object_model::OBJECT_REF_OFFSET;

use crate::{mmtk, OCaml4VM, SINGLETON};

// ── Init ──────────────────────────────────────────────────────────────────

/// Initialise MMTk with a fixed heap size and GC plan name.
///
/// Must be called exactly once before any allocation.
/// `plan` is a null-terminated C string: "NoGC", "MarkSweep", "Immix", etc.
#[no_mangle]
pub extern "C" fn mmtk_init(heap_size: usize, plan: *const libc::c_char) {
    let plan_str = unsafe { CStr::from_ptr(plan).to_str().expect("invalid plan string") };

    let mut builder = MMTKBuilder::new();
    assert!(
        memory_manager::process(&mut builder, "plan", plan_str),
        "unknown MMTk plan: {}", plan_str
    );
    let gc_trigger_val = format!("FixedHeapSize:{}", heap_size);
    assert!(
        memory_manager::process(&mut builder, "gc_trigger", &gc_trigger_val),
        "failed to set gc_trigger (heap_size)"
    );

    let mmtk_instance = memory_manager::mmtk_init::<OCaml4VM>(&builder);
    SINGLETON
        .set(mmtk_instance)
        .ok()
        .expect("mmtk_init called more than once");
}

// ── Collection initialisation ─────────────────────────────────────────────

/// Start MMTk GC worker threads.  Call once after mmtk_init and before any allocation.
///
/// `tls` — the calling thread's ID (e.g. pthread_self() cast to usize).
#[no_mangle]
pub extern "C" fn mmtk_initialize_collection(tls: usize) {
    let tls = VMThread(OpaquePointer::from_address(
        unsafe { Address::from_usize(tls) },
    ));
    memory_manager::initialize_collection::<OCaml4VM>(mmtk(), tls);
}

// ── Mutator lifecycle ─────────────────────────────────────────────────────

/// Bind the calling OCaml 4.14 thread as an MMTk mutator.
/// Returns an opaque mutator handle stored in the thread's caml_domain_state.
///
/// `tls` — platform thread identifier (e.g. pthread_t cast to usize).
#[no_mangle]
pub extern "C" fn mmtk_bind_mutator(tls: usize) -> *mut libc::c_void {
    let tls = VMMutatorThread(VMThread(OpaquePointer::from_address(
        unsafe { Address::from_usize(tls) },
    )));
    let mutator = memory_manager::bind_mutator(mmtk(), tls);
    // TODO: register mutator in active_plan::MUTATOR_REGISTRY
    Box::into_raw(mutator) as *mut libc::c_void
}

/// Unbind and destroy the calling thread's mutator.
#[no_mangle]
pub extern "C" fn mmtk_destroy_mutator(mutator: *mut libc::c_void) {
    // TODO: deregister from active_plan::MUTATOR_REGISTRY
    // Reconstruct the Box so the allocation is freed after destroy_mutator runs.
    let mut mutator_box = unsafe { Box::from_raw(mutator as *mut mmtk::Mutator<OCaml4VM>) };
    memory_manager::destroy_mutator(&mut *mutator_box);
    // mutator_box drops here, freeing the Mutator allocation.
}

// ── Allocation ────────────────────────────────────────────────────────────

/// Allocate an OCaml block with `wosize` fields and the given `tag`.
///
/// Returns a pointer to field 0 (the OCaml object reference convention).
/// The header word immediately precedes the returned pointer.
///
/// `semantics` — 0 for default (uses LOS for large objects automatically).
#[no_mangle]
pub extern "C" fn mmtk_alloc(
    mutator: *mut libc::c_void,
    wosize: usize,
    tag: usize,
    semantics: usize,
) -> *mut libc::c_void {
    let mutator = unsafe { &mut *(mutator as *mut mmtk::Mutator<OCaml4VM>) };
    let total_bytes = (wosize + 1) * WORD_SIZE; // header + fields
    let semantics = match semantics {
        0 => AllocationSemantics::Default,
        1 => AllocationSemantics::Immortal,
        2 => AllocationSemantics::Los,
        6 => AllocationSemantics::NonMoving,
        _ => AllocationSemantics::Default,
    };

    let alloc_start: Address =
        memory_manager::alloc::<OCaml4VM>(mutator, total_bytes, WORD_SIZE, 0, semantics);

    // Write the header word at alloc_start.
    let header = make_header(wosize, tag as u8);
    unsafe { alloc_start.store(header) };

    // Object reference is one word past the header.
    let obj_ref = alloc_start + OBJECT_REF_OFFSET;

    // Inform MMTk that the object has been placed.
    let object =
        unsafe { ObjectReference::from_raw_address_unchecked(obj_ref) };
    memory_manager::post_alloc::<OCaml4VM>(mutator, object, total_bytes, semantics);

    eprintln!("[mmtk-ocaml4] alloc wosize={} tag={} → 0x{:x}", wosize, tag, obj_ref.as_usize());
    obj_ref.to_mut_ptr::<libc::c_void>()
}

// ── GC control ───────────────────────────────────────────────────────────

/// Request an immediate (non-concurrent) GC cycle.
#[no_mangle]
pub extern "C" fn mmtk_handle_user_collection_request(tls: usize) {
    let tls = VMMutatorThread(VMThread(OpaquePointer::from_address(
        unsafe { Address::from_usize(tls) },
    )));
    memory_manager::handle_user_collection_request::<OCaml4VM>(mmtk(), tls);
}

// ── Object queries ────────────────────────────────────────────────────────

/// True if `addr` points into an MMTk-managed heap region.
#[no_mangle]
pub extern "C" fn mmtk_is_in_mmtk_spaces(addr: *const libc::c_void) -> bool {
    let addr = Address::from_ptr(addr);
    memory_manager::is_in_mmtk_spaces(unsafe {
        ObjectReference::from_raw_address_unchecked(addr)
    })
}
