//! VMScanning for OCaml 5.x — root enumeration and object tracing.
//!
//! Root enumeration hook:
//!   OCaml 5.x provides:
//!     caml_do_roots(scanning_action f, scanning_action_flags flags,
//!                   void* fdata, caml_domain_state* domain, int do_final)
//!   The scanning_action callback:
//!     void callback(void* fdata, value v, volatile value* slot_addr)
//!   We collect all `slot_addr` pointers as FieldSlots and pass to MMTk.
//!
//!   `scan_roots_in_mutator_thread` is called once per domain (= mutator) during STW.
//!   We obtain the `caml_domain_state*` from the mutator TLS and call `caml_do_roots`.
//!
//! Global roots in OCaml 5.x:
//!   `caml_global_roots` list (same as 4.14).
//!   Per-domain `extern_state` and `backtrace_last_exn` also need scanning.
//!
//! TODO (agent): implement scan_roots_in_mutator_thread and scan_vm_specific_roots.

use mmtk::util::opaque_pointer::VMWorkerThread;
use mmtk::util::ObjectReference;
use mmtk::vm::SlotVisitor;
use mmtk::vm::{RootsWorkFactory, Scanning};
use mmtk::Mutator;

use crate::header::{tag_of, wosize_of, TAG_CLOSURE, TAG_FORWARD, TAG_INFIX, TAG_NO_SCAN, WORD_SIZE};
use crate::slot::FieldSlot;
use crate::OCaml5VM;

/// Visit all GC-visible pointer fields of an OCaml heap block.
///
/// The caller is responsible for ensuring `object` is a valid, live OCaml block
/// (i.e. not a tagged integer and not null).
fn scan_ocaml_object<SV: SlotVisitor<FieldSlot>>(
    object: ObjectReference,
    slot_visitor: &mut SV,
) {
    let base = object.to_raw_address();

    let header: usize = unsafe { (base - WORD_SIZE).load() };
    let tag = tag_of(header);
    let wosize = wosize_of(header);

    // Tags >= TAG_NO_SCAN (Abstract, String, Double, Double_array, Custom) carry
    // no GC-visible pointer fields; nothing to visit.
    if tag >= TAG_NO_SCAN {
        return;
    }

    match tag {
        TAG_INFIX => {
            // An infix block is an interior pointer into a closure.  The parent
            // closure will itself be reached and scanned via its own object reference,
            // so we skip the infix block entirely.
            //
            // TODO(infix-redirect): moving/compacting GC must redirect the infix pointer
            // after the parent closure moves.  Implement in copy_object when adding
            // Immix defrag.
        }

        TAG_CLOSURE => {
            // Field 0 is a raw code pointer (address into .text section).
            // It is *not* a GC root and must not be treated as one.
            // Scan fields 1..wosize (the closure environment).
            //
            // TODO(closure-multientry): multi-entry closures (arity > 1) have an arity
            // word and additional code-pointer slots interspersed — audit once we have
            // closures in benchmarks.
            for i in 1..wosize {
                let slot_addr = base + i * WORD_SIZE;
                slot_visitor.visit_slot(FieldSlot::from_address(slot_addr));
            }
        }

        TAG_FORWARD => {
            // Forwarding pointer: field 0 holds the new location of a moved object.
            // Visit it so MMTk can update the chain if the target also moves.
            if wosize > 0 {
                slot_visitor.visit_slot(FieldSlot::from_address(base));
            }
        }

        _ => {
            // Ordinary block (tag 0..245), Lazy (246), Object (248):
            // all fields are OCaml values — each may be an immediate int or
            // a heap pointer.  FieldSlot::load() filters out immediates.
            for i in 0..wosize {
                let slot_addr = base + i * WORD_SIZE;
                slot_visitor.visit_slot(FieldSlot::from_address(slot_addr));
            }
        }
    }
}

pub struct VMScanning;

impl Scanning<OCaml5VM> for VMScanning {
    /// Enumerate all GC roots for a single OCaml 5.x domain.
    ///
    /// Implementation plan:
    ///   1. Recover the caml_domain_state* from the mutator's TLS address.
    ///   2. Declare a C callback that appends slot addresses to Vec<FieldSlot>.
    ///   3. Call caml_do_roots(callback, SCANNING_ONLY_YOUNG_VALUES|0, &mut vec,
    ///                         domain_state, /*do_final=*/0).
    ///   4. factory.create_process_roots_work(vec).
    ///
    /// Note: caml_do_roots on 5.x also walks the domain's fiber/effect stacks.
    fn scan_roots_in_mutator_thread(
        _tls: VMWorkerThread,
        _mutator: &'static mut Mutator<OCaml5VM>,
        _factory: impl RootsWorkFactory<FieldSlot>,
    ) {
        todo!(
            "OCaml 5.x scan_roots_in_mutator_thread: \
             call caml_do_roots per domain with a FieldSlot-collecting callback"
        )
    }

    /// Enumerate global roots not owned by any specific domain.
    ///
    /// Implementation plan:
    ///   Walk caml_global_roots.  In OCaml 5.x also check caml_shared_heap roots.
    fn scan_vm_specific_roots(
        _tls: VMWorkerThread,
        _factory: impl RootsWorkFactory<FieldSlot>,
    ) {
        todo!(
            "OCaml 5.x scan_vm_specific_roots: \
             enumerate caml_global_roots and any shared heap roots"
        )
    }

    /// Trace all pointer fields of a live OCaml heap block.
    fn scan_object<SV: SlotVisitor<FieldSlot>>(
        _tls: VMWorkerThread,
        object: ObjectReference,
        slot_visitor: &mut SV,
    ) {
        scan_ocaml_object(object, slot_visitor);
    }

    fn notify_initial_thread_scan_complete(_partial_scan: bool, _tls: VMWorkerThread) {}

    fn supports_return_barrier() -> bool {
        false
    }

    fn prepare_for_roots_re_scanning() {}
}
