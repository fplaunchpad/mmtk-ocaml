use mmtk::util::copy::{CopySemantics, GCWorkerCopyContext};
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::ObjectModel;
use mmtk::vm::{
    VMGlobalLogBitSpec, VMLocalForwardingBitsSpec, VMLocalForwardingPointerSpec,
    VMLocalLOSMarkNurserySpec, VMLocalMarkBitSpec, VMLocalPinningBitSpec,
};

use mmtk_ocaml_common::header::WORD_SIZE;
use mmtk_ocaml_common::object_model as common;

use crate::OCaml4VM;

pub struct VMObjectModel;

impl ObjectModel<OCaml4VM> for VMObjectModel {
    // ── Metadata placement ────────────────────────────────────────────────
    // All metadata uses side tables so the OCaml header word is never overwritten
    // during normal operation.  The forwarding pointer is an exception: on object
    // copy the entire header is replaced with the forwarding address (safe because
    // the forwarding-bits spec tells MMTk whether to interpret the header as a
    // forwarding pointer or as an OCaml header).

    const GLOBAL_LOG_BIT_SPEC: VMGlobalLogBitSpec =
        VMGlobalLogBitSpec::side_first();

    const LOCAL_FORWARDING_POINTER_SPEC: VMLocalForwardingPointerSpec =
        VMLocalForwardingPointerSpec::in_header(0);

    const LOCAL_FORWARDING_BITS_SPEC: VMLocalForwardingBitsSpec =
        VMLocalForwardingBitsSpec::side_first();

    const LOCAL_MARK_BIT_SPEC: VMLocalMarkBitSpec =
        VMLocalMarkBitSpec::side_after(Self::LOCAL_FORWARDING_BITS_SPEC.as_spec());

    const LOCAL_LOS_MARK_NURSERY_SPEC: VMLocalLOSMarkNurserySpec =
        VMLocalLOSMarkNurserySpec::side_after(Self::LOCAL_MARK_BIT_SPEC.as_spec());

    const LOCAL_PINNING_BIT_SPEC: VMLocalPinningBitSpec =
        VMLocalPinningBitSpec::side_after(Self::LOCAL_LOS_MARK_NURSERY_SPEC.as_spec());

    // Object reference is WORD_SIZE bytes into the allocation result (past the header).
    const OBJECT_REF_OFFSET_LOWER_BOUND: isize = common::OBJECT_REF_OFFSET as isize;

    // ── Address layout ────────────────────────────────────────────────────

    fn ref_to_object_start(object: ObjectReference) -> Address {
        common::ref_to_object_start(object)
    }

    fn ref_to_header(object: ObjectReference) -> Address {
        common::ref_to_header(object)
    }

    // ── Size ──────────────────────────────────────────────────────────────

    fn get_current_size(object: ObjectReference) -> usize {
        common::get_current_size(object)
    }

    fn get_size_when_copied(object: ObjectReference) -> usize {
        common::get_current_size(object)
    }

    fn get_align_when_copied(_object: ObjectReference) -> usize {
        WORD_SIZE
    }

    fn get_align_offset_when_copied(_object: ObjectReference) -> usize {
        0
    }

    // ── Copy ──────────────────────────────────────────────────────────────

    fn copy(
        from: ObjectReference,
        semantics: CopySemantics,
        copy_context: &mut GCWorkerCopyContext<OCaml4VM>,
    ) -> ObjectReference {
        common::copy_object::<OCaml4VM>(from, semantics, copy_context)
    }

    fn copy_to(from: ObjectReference, to: ObjectReference, region: Address) -> Address {
        common::copy_to_object(from, to, region)
    }

    fn get_reference_when_copied_to(from: ObjectReference, to: Address) -> ObjectReference {
        common::get_reference_when_copied_to(from, to)
    }

    // ── Misc ──────────────────────────────────────────────────────────────

    fn get_type_descriptor(_reference: ObjectReference) -> &'static [i8] {
        &[] // unused; OCaml does not expose RTTI through MMTk
    }

    fn dump_object(object: ObjectReference) {
        let header: usize = unsafe { common::ref_to_header(object).load() };
        eprintln!(
            "OCaml4 object @ {:?}: wosize={} tag={}",
            object,
            header >> 10,
            header & 0xFF
        );
    }
}
