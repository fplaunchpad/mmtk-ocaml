//! VMObjectModel for OCaml 5.x.
//!
//! Object layout in memory:
//!
//!   alloc result  →  [ header word ][ field 0 ][ field 1 ] … [ field N-1 ]
//!   object ref    →                 ^ (one WORD_SIZE past alloc result)

use mmtk::util::copy::{CopySemantics, GCWorkerCopyContext};
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::ObjectModel;
use mmtk::vm::{
    VMGlobalLogBitSpec, VMLocalForwardingBitsSpec, VMLocalForwardingPointerSpec,
    VMLocalLOSMarkNurserySpec, VMLocalMarkBitSpec, VMLocalPinningBitSpec,
};

use crate::header::{tag_of, wosize_of, WORD_SIZE};
use crate::OCaml5VM;

/// Byte offset from the MMTk allocation result to the OCaml object reference.
pub const OBJECT_REF_OFFSET: usize = WORD_SIZE;

/// Read the header word of a live OCaml object.
///
/// # Safety boundary
/// `object` must be a live `ObjectReference` inside MMTk's managed heap.
/// `ObjectReference` guarantees non-null (`NonZeroUsize`) and `WORD_SIZE`
/// alignment, so `object.to_raw_address() - WORD_SIZE` is always a valid,
/// readable address by OCaml's `Val_hp` layout invariant.
#[inline(always)]
pub fn read_header(object: ObjectReference) -> usize {
    unsafe { (object.to_raw_address() - OBJECT_REF_OFFSET).load() }
}

pub struct VMObjectModel;

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

    const OBJECT_REF_OFFSET_LOWER_BOUND: isize = OBJECT_REF_OFFSET as isize;

    fn ref_to_object_start(object: ObjectReference) -> Address {
        object.to_raw_address() - OBJECT_REF_OFFSET
    }

    fn ref_to_header(object: ObjectReference) -> Address {
        Self::ref_to_object_start(object)
    }

    fn get_current_size(object: ObjectReference) -> usize {
        (wosize_of(read_header(object)) + 1) * WORD_SIZE
    }

    fn get_size_when_copied(object: ObjectReference) -> usize {
        Self::get_current_size(object)
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
        let size = VMObjectModel::get_current_size(from);
        let from_start = VMObjectModel::ref_to_object_start(from);

        let to_start = copy_context.alloc_copy(from, size, WORD_SIZE, 0, semantics);
        assert!(
            !to_start.is_zero(),
            "alloc_copy returned null for object {:#x} ({} bytes, semantics {:?}): \
             tospace exhausted — heap is full and evacuation cannot complete",
            from.to_raw_address().as_usize(),
            size,
            semantics,
        );
        // TODO(oom-protocol): Replace assert with proper two-way OOM protocol.
        // GC worker should signal the mutator to raise OCaml Out_of_memory rather
        // than aborting.  Requires VMCollection::out_of_memory and a cross-thread
        // flag on the mutator handle checked at the next safepoint.

        // SAFETY: from_start is (non-null, word-aligned): derived from ObjectReference.
        // to_start is (non-null, word-aligned): asserted above + alloc_copy contract.
        unsafe {
            std::ptr::copy_nonoverlapping(from_start.to_ptr::<u8>(), to_start.to_mut_ptr::<u8>(), size);
        }

        // SAFETY: to_start non-null and word-aligned; adding WORD_SIZE preserves both.
        let to_ref = unsafe {
            ObjectReference::from_raw_address_unchecked(to_start + OBJECT_REF_OFFSET)
        };
        copy_context.post_copy(to_ref, size, semantics);
        to_ref
    }

    fn copy_to(from: ObjectReference, to: ObjectReference, _region: Address) -> Address {
        // _region is always Address::ZERO in both current MMTk callers (MarkCompact,
        // Compressor); destination is derived from `to` inside copy_to_object.
        let size = VMObjectModel::get_current_size(from);
        let from_start = VMObjectModel::ref_to_object_start(from);
        let dst = VMObjectModel::ref_to_object_start(to);
        // SAFETY: both from_start and dst come from ObjectReference, which guarantees
        // non-null and WORD_SIZE alignment; ref_to_object_start subtracts WORD_SIZE,
        // preserving both.
        unsafe {
            std::ptr::copy_nonoverlapping(from_start.to_ptr::<u8>(), dst.to_mut_ptr::<u8>(), size);
        }
        dst + size
    }

    fn get_reference_when_copied_to(_from: ObjectReference, to: Address) -> ObjectReference {
        debug_assert!(
            !to.is_zero() && to.is_aligned_to(WORD_SIZE),
            "get_reference_when_copied_to: invalid region address {:#x} (null or misaligned)",
            to.as_usize()
        );
        // SAFETY: `to` is validated non-null and WORD_SIZE-aligned above.
        // Adding OBJECT_REF_OFFSET preserves both properties.
        unsafe { ObjectReference::from_raw_address_unchecked(to + OBJECT_REF_OFFSET) }
    }

    fn get_type_descriptor(_reference: ObjectReference) -> &'static [i8] {
        &[] // Vestigial MMTk hook — no equivalent in OCaml.
    }

    fn dump_object(object: ObjectReference) {
        let header = read_header(object);
        eprintln!(
            "OCaml object @ {:#x}: wosize={} tag={}",
            object.to_raw_address().as_usize(),
            wosize_of(header),
            tag_of(header),
        );
    }
}
