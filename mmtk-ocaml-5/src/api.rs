//! C-exported API for the OCaml 5.x MMTk binding.
//!
//! Mirrors mmtk-ocaml-4/src/api.rs but uses domain-based TLS.
//! Key difference: `tls` is the address of `caml_domain_state*` (not a pthread_t).
//!
//! Function naming convention: mmtk_ocaml5_*  (avoids link-time conflicts when
//! both libraries are present in the same test harness).
//!
//! TODO (agent): implement each function — same bring-up sequence as OCaml 4.14.

use std::ffi::CStr;

use mmtk::memory_manager;
use mmtk::util::opaque_pointer::{OpaquePointer, VMMutatorThread, VMThread};
use mmtk::util::{Address, ObjectReference};
use mmtk::AllocationSemantics;
use mmtk::MMTKBuilder;

use mmtk_ocaml_common::header::{make_header, WORD_SIZE};
use mmtk_ocaml_common::object_model::OBJECT_REF_OFFSET;

use crate::active_plan::{register_mutator, deregister_by_ptr};
use crate::{mmtk, OCaml5VM, SINGLETON};

// ── Init ──────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn mmtk_ocaml5_init(heap_size: usize, plan: *const libc::c_char) {
    let plan_str = unsafe { CStr::from_ptr(plan).to_str().expect("invalid plan string") };

    let mut builder = MMTKBuilder::new();
    assert!(
        memory_manager::process(&mut builder, "plan", plan_str),
        "unknown MMTk plan: {}", plan_str
    );
    assert!(
        memory_manager::process(&mut builder, "gc_trigger", &format!("FixedHeapSize:{}", heap_size)),
        "failed to set gc_trigger/heap_size"
    );

    let mmtk_instance = memory_manager::mmtk_init::<OCaml5VM>(&builder);
    SINGLETON
        .set(mmtk_instance)
        .ok()
        .expect("mmtk_ocaml5_init called more than once");
}

/// Start MMTk GC worker threads.  Call once after mmtk_ocaml5_init, before any allocation.
#[no_mangle]
pub extern "C" fn mmtk_ocaml5_initialize_collection(tls: usize) {
    let tls = VMThread(OpaquePointer::from_address(
        unsafe { Address::from_usize(tls) },
    ));
    memory_manager::initialize_collection::<OCaml5VM>(mmtk(), tls);
}

// ── Mutator (domain) lifecycle ────────────────────────────────────────────

/// Bind a new OCaml 5.x domain as an MMTk mutator.
/// `domain_state_addr` — the address of the domain's caml_domain_state struct.
#[no_mangle]
pub extern "C" fn mmtk_ocaml5_bind_mutator(domain_state_addr: usize) -> *mut libc::c_void {
    let tls = VMMutatorThread(VMThread(OpaquePointer::from_address(
        unsafe { Address::from_usize(domain_state_addr) },
    )));
    let mutator = memory_manager::bind_mutator(mmtk(), tls);
    let raw = Box::into_raw(mutator);
    register_mutator(domain_state_addr, raw);
    raw as *mut libc::c_void
}

/// Destroy the mutator for a terminating OCaml 5.x domain.
#[no_mangle]
pub extern "C" fn mmtk_ocaml5_destroy_mutator(mutator: *mut libc::c_void) {
    let mutator_ptr = mutator as *mut mmtk::Mutator<OCaml5VM>;
    deregister_by_ptr(mutator_ptr);
    // Reconstruct the Box so the allocation is freed after destroy_mutator runs.
    let mut mutator_box = unsafe { Box::from_raw(mutator_ptr) };
    memory_manager::destroy_mutator(&mut *mutator_box);
    // mutator_box drops here, freeing the Mutator allocation.
}

// ── Allocation ────────────────────────────────────────────────────────────

/// Allocate an OCaml block.  Same semantics as the OCaml 4.14 version.
#[no_mangle]
pub extern "C" fn mmtk_ocaml5_alloc(
    mutator: *mut libc::c_void,
    wosize: usize,
    tag: usize,
    semantics: usize,
) -> *mut libc::c_void {
    let mutator = unsafe { &mut *(mutator as *mut mmtk::Mutator<OCaml5VM>) };
    let total_bytes = (wosize + 1) * WORD_SIZE;
    let semantics = match semantics {
        0 => AllocationSemantics::Default,
        1 => AllocationSemantics::Immortal,
        2 => AllocationSemantics::Los,
        6 => AllocationSemantics::NonMoving,
        _ => AllocationSemantics::Default,
    };

    let alloc_start: Address =
        memory_manager::alloc::<OCaml5VM>(mutator, total_bytes, WORD_SIZE, 0, semantics);

    let header = make_header(wosize, tag as u8);
    unsafe { alloc_start.store(header) };

    let obj_ref = alloc_start + OBJECT_REF_OFFSET;
    let object = unsafe { ObjectReference::from_raw_address_unchecked(obj_ref) };
    memory_manager::post_alloc::<OCaml5VM>(mutator, object, total_bytes, semantics);

    eprintln!("[mmtk-ocaml5] alloc wosize={} tag={} → 0x{:x}", wosize, tag, obj_ref.as_usize());
    obj_ref.to_mut_ptr::<libc::c_void>()
}

// ── GC control ────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn mmtk_ocaml5_handle_user_collection_request(domain_state_addr: usize) {
    let tls = VMMutatorThread(VMThread(OpaquePointer::from_address(
        unsafe { Address::from_usize(domain_state_addr) },
    )));
    memory_manager::handle_user_collection_request::<OCaml5VM>(mmtk(), tls);
}

// ── Object queries ────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn mmtk_ocaml5_is_in_mmtk_spaces(addr: *const libc::c_void) -> bool {
    let addr = Address::from_ptr(addr);
    memory_manager::is_in_mmtk_spaces(unsafe {
        ObjectReference::from_raw_address_unchecked(addr)
    })
}
