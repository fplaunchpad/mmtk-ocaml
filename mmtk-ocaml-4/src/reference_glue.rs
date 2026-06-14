//! VMReferenceGlue for OCaml 4.14 — weak references and finalizers.
//!
//! OCaml's finalization model (runtime/finalise.c):
//!   Objects are registered with `caml_register_final` / `caml_final_register`.
//!   The GC calls `caml_final_do_roots` during marking to keep finalizable objects
//!   alive, then enqueues them via `caml_final_do_calls` after collection.
//!
//! For the initial NoGC / MarkSweep phases, finalization is left as unimplemented.
//! Implement once basic GC is working.
//!
//! TODO (agent): implement once basic collection is working.

use mmtk::util::opaque_pointer::VMWorkerThread;
use mmtk::util::ObjectReference;
use mmtk::vm::ReferenceGlue;

use crate::OCaml4VM;

pub struct VMReferenceGlue;

impl ReferenceGlue<OCaml4VM> for VMReferenceGlue {
    type FinalizableType = ObjectReference;

    fn clear_referent(_new_reference: ObjectReference) {
        unimplemented!("OCaml 4.14 finalizers — implement after basic GC works")
    }

    fn get_referent(_object: ObjectReference) -> Option<ObjectReference> {
        unimplemented!()
    }

    fn set_referent(_reff: ObjectReference, _referent: ObjectReference) {
        unimplemented!()
    }

    fn enqueue_references(_references: &[ObjectReference], _tls: VMWorkerThread) {
        unimplemented!()
    }
}
