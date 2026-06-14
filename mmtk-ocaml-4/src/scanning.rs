//! VMScanning for OCaml 4.14 — root enumeration and object tracing.
//!
//! Root enumeration hook:
//!   OCaml 4.14 provides `caml_do_roots(scanning_action f, void* fdata, domain_state* d)`.
//!   The scanning_action callback signature is:
//!     void callback(void* fdata, value v, volatile value* slot_addr)
//!   We collect all `slot_addr` pointers and hand them to MMTk as FieldSlots.
//!
//!   Entry point: `scan_roots_in_mutator_thread` is called once per mutator thread
//!   during STW.  We call `caml_do_roots` for that thread's domain_state.
//!
//! Global roots:
//!   `caml_global_roots` and `caml_global_roots_young` are C-side lists of
//!   pointers registered via `caml_register_global_root`.  We enumerate them in
//!   `scan_vm_specific_roots`.
//!
//! TODO (agent): implement scan_roots_in_mutator_thread and scan_vm_specific_roots.

use mmtk::util::opaque_pointer::{VMMutatorThread, VMWorkerThread};
use mmtk::util::ObjectReference;
use mmtk::vm::SlotVisitor;
use mmtk::vm::{RootsWorkFactory, Scanning};
use mmtk::Mutator;

use mmtk_ocaml_common::scanning::scan_ocaml_object;
use mmtk_ocaml_common::slot::FieldSlot;

use crate::OCaml4VM;

pub struct VMScanning;

impl Scanning<OCaml4VM> for VMScanning {
    /// Enumerate all GC roots reachable from a single OCaml 4.14 thread.
    ///
    /// Implementation plan:
    ///   1. Obtain the domain_state* for `mutator` (stored during bind_mutator).
    ///   2. Declare a C callback that appends the slot address to a Vec<FieldSlot>.
    ///   3. Call `caml_do_roots(callback, &mut slots_vec, domain_state, /*do_final=*/0)`.
    ///   4. `factory.create_process_roots_work(slots_vec)`.
    ///
    /// caml_do_roots covers: CAMLlocal/CAMLparam roots, native stack (via frame tables),
    /// and domain-local finaliser roots.  Global roots are handled separately.
    fn scan_roots_in_mutator_thread(
        _tls: VMWorkerThread,
        _mutator: &'static mut Mutator<OCaml4VM>,
        _factory: impl RootsWorkFactory<FieldSlot>,
    ) {
        todo!(
            "OCaml 4.14 scan_roots_in_mutator_thread: \
             call caml_do_roots with a FieldSlot-collecting callback"
        )
    }

    /// Enumerate global / VM-specific roots not associated with any one thread.
    ///
    /// Implementation plan:
    ///   Walk `caml_global_roots` and `caml_global_roots_young` linked lists.
    ///   For each entry, push the address of the root slot as a FieldSlot.
    fn scan_vm_specific_roots(
        _tls: VMWorkerThread,
        _factory: impl RootsWorkFactory<FieldSlot>,
    ) {
        todo!(
            "OCaml 4.14 scan_vm_specific_roots: \
             enumerate caml_global_roots and caml_global_roots_young"
        )
    }

    /// Trace all pointer fields of a live OCaml heap block.
    /// Delegates to the shared implementation in mmtk-ocaml-common.
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
