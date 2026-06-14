//! Shared VMObjectModel helpers for both OCaml 4.14 and 5.x.
//!
//! Object layout in memory:
//!
//!   alloc result  →  [ header word ][ field 0 ][ field 1 ] … [ field N-1 ]
//!   object ref    →                 ^ (one WORD_SIZE past alloc result)
//!
//! The object reference points to field 0; the header is one word before it.
//! MMTk's `ref_to_object_start` must return the header address (= alloc result).

use mmtk::util::copy::{CopySemantics, GCWorkerCopyContext};
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::VMBinding;

use crate::header::{wosize_of, WORD_SIZE};

/// Byte offset from the MMTk allocation result to the OCaml object reference.
pub const OBJECT_REF_OFFSET: usize = WORD_SIZE;

// ── Address helpers ───────────────────────────────────────────────────────

/// Address of the header word (= alloc result = object start).
#[inline(always)]
pub fn ref_to_header(object: ObjectReference) -> Address {
    object.to_raw_address() - OBJECT_REF_OFFSET
}

/// Address of the first allocated byte (same as the header for OCaml).
#[inline(always)]
pub fn ref_to_object_start(object: ObjectReference) -> Address {
    object.to_raw_address() - OBJECT_REF_OFFSET
}

// ── Size ──────────────────────────────────────────────────────────────────

/// Total allocated size of an object in bytes: header word + all fields.
///
/// Reads Wosize from the object's header to compute the field count.
#[inline(always)]
pub fn get_current_size(object: ObjectReference) -> usize {
    let header: usize = unsafe { ref_to_header(object).load() };
    let wosize = wosize_of(header);
    (wosize + 1) * WORD_SIZE // +1 for the header word itself
}

// ── Copy ─────────────────────────────────────────────────────────────────

/// Copy `object` to a new location allocated through `copy_context`.
/// Returns the new `ObjectReference`.
///
/// Used by Immix defragmentation, SemiSpace, GenCopy, and any other
/// copying/moving GC plan.
pub fn copy_object<VM: VMBinding>(
    from: ObjectReference,
    semantics: CopySemantics,
    copy_context: &mut GCWorkerCopyContext<VM>,
) -> ObjectReference {
    let size = get_current_size(from);
    let to_start = copy_context.alloc_copy(from, size, WORD_SIZE, 0, semantics);

    // Bulk-copy header + all fields to the new location.
    unsafe {
        std::ptr::copy_nonoverlapping(
            ref_to_object_start(from).to_ptr::<u8>(),
            to_start.to_mut_ptr::<u8>(),
            size,
        );
    }

    // The new object reference is one word past the new allocation start.
    let to_ref = unsafe {
        ObjectReference::from_raw_address_unchecked(to_start + OBJECT_REF_OFFSET)
    };
    copy_context.post_copy(to_ref, size, semantics);
    to_ref
}

/// Copy-to variant used by delayed-copy (compacting) collectors.
/// Returns the address past the end of the copied object.
pub fn copy_to_object(
    from: ObjectReference,
    _to: ObjectReference,
    region: Address,
) -> Address {
    let size = get_current_size(from);
    unsafe {
        std::ptr::copy_nonoverlapping(
            ref_to_object_start(from).to_ptr::<u8>(),
            region.to_mut_ptr::<u8>(),
            size,
        );
    }
    region + size
}

/// Predict where the object reference will be once the object is copied
/// to `to` (start of reserved region).
#[inline(always)]
pub fn get_reference_when_copied_to(_from: ObjectReference, to: Address) -> ObjectReference {
    unsafe { ObjectReference::from_raw_address_unchecked(to + OBJECT_REF_OFFSET) }
}
