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

use mmtk::util::opaque_pointer::{VMMutatorThread, VMWorkerThread};
use mmtk::util::ObjectReference;
use mmtk::vm::SlotVisitor;
use mmtk::vm::{RootsWorkFactory, Scanning};
use mmtk::Mutator;

use mmtk_ocaml_common::scanning::scan_ocaml_object;
use mmtk_ocaml_common::slot::FieldSlot;

use crate::OCaml5VM;

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
    /// Identical to OCaml 4.14 — the block format is unchanged.
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
