//! MMTk binding for OCaml 4.14 LTS.
//!
//! Architecture: one mutator per OS thread; STW via caml_young_limit flip;
//! roots via caml_do_roots (single domain).
//!
//! Agents working in this worktree should only edit files under mmtk-ocaml-4/.
//! Shared types live in mmtk-ocaml-common/ — read-only from this crate.

use std::sync::OnceLock;

use mmtk::vm::VMBinding;
use mmtk::MMTK;

pub mod active_plan;
pub mod api;
pub mod collection;
pub mod object_model;
pub mod reference_glue;
pub mod scanning;

/// The OCaml 4.14 VM tag — zero-sized, used only as a type parameter.
#[derive(Default)]
pub struct OCaml4VM;

impl VMBinding for OCaml4VM {
    type VMObjectModel   = object_model::VMObjectModel;
    type VMScanning      = scanning::VMScanning;
    type VMCollection    = collection::VMCollection;
    type VMActivePlan    = active_plan::VMActivePlan;
    type VMReferenceGlue = reference_glue::VMReferenceGlue;
    type VMSlot          = mmtk_ocaml_common::slot::FieldSlot;
    type VMMemorySlice   = mmtk_ocaml_common::slot::UnimplementedMemorySlice;

    // Every OCaml allocation requests WORD_SIZE alignment and offset=0.
    // MIN = MAX = WORD_SIZE: no alignment padding ever needed for copies.
    // USE_ALLOCATION_OFFSET = false: we always pass offset=0, lets MMTk skip
    // the offset branch in the allocator fast path.
    const MIN_ALIGNMENT: usize = mmtk_ocaml_common::header::WORD_SIZE;
    const MAX_ALIGNMENT: usize = mmtk_ocaml_common::header::WORD_SIZE;
    const USE_ALLOCATION_OFFSET: bool = false;
}

/// The global MMTk instance.  Initialised once by `mmtk_init` in api.rs.
pub static SINGLETON: OnceLock<Box<MMTK<OCaml4VM>>> = OnceLock::new();

/// Convenience accessor — panics if called before `mmtk_init`.
pub fn mmtk() -> &'static MMTK<OCaml4VM> {
    SINGLETON.get().expect("MMTk not initialised — call mmtk_init first")
}
