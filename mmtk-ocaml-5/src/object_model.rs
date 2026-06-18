use mmtk::util::copy::{CopySemantics, GCWorkerCopyContext};
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::ObjectModel;
use mmtk::vm::{
    VMGlobalLogBitSpec, VMLocalForwardingBitsSpec, VMLocalForwardingPointerSpec,
    VMLocalLOSMarkNurserySpec, VMLocalMarkBitSpec, VMLocalPinningBitSpec,
};

use mmtk_ocaml_common::header::WORD_SIZE;
use mmtk_ocaml_common::object_model as common;

use crate::OCaml5VM;

pub struct VMObjectModel;

// Identical to OCaml 4.14 — the block header format did not change in OCaml 5.
impl ObjectModel<OCaml5VM> for VMObjectModel {
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

    const OBJECT_REF_OFFSET_LOWER_BOUND: isize = common::OBJECT_REF_OFFSET as isize;

    fn ref_to_object_start(object: ObjectReference) -> Address {
        common::ref_to_object_start(object)
    }

    fn ref_to_header(object: ObjectReference) -> Address {
        common::ref_to_header(object)
    }

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

    fn copy(
        from: ObjectReference,
        semantics: CopySemantics,
        copy_context: &mut GCWorkerCopyContext<OCaml5VM>,
    ) -> ObjectReference {
        common::copy_object::<OCaml5VM>(from, semantics, copy_context)
    }

    fn copy_to(from: ObjectReference, to: ObjectReference, _region: Address) -> Address {
        // _region is always Address::ZERO in both current MMTk callers (MarkCompact,
        // Compressor); the destination is derived from `to` inside copy_to_object.
        common::copy_to_object(from, to)
    }

    fn get_reference_when_copied_to(from: ObjectReference, to: Address) -> ObjectReference {
        common::get_reference_when_copied_to(from, to)
    }

    fn get_type_descriptor(_reference: ObjectReference) -> &'static [i8] {
        &[]
    }

    fn dump_object(object: ObjectReference) {
        common::dump_object(object);
    }
}
