//! VMReferenceGlue for OCaml 5.x — weak references and finalizers.
//!
//! Same semantics as OCaml 4.14 (runtime/finalise.c is largely unchanged),
//! except each domain has its own finaliser queue.
//!
//! TODO (agent): implement once basic per-domain GC is working.

use mmtk::util::opaque_pointer::VMWorkerThread;
use mmtk::util::ObjectReference;
use mmtk::vm::ReferenceGlue;

use crate::OCaml5VM;

pub struct VMReferenceGlue;

impl ReferenceGlue<OCaml5VM> for VMReferenceGlue {
    type FinalizableType = ObjectReference;

    fn clear_referent(_new_reference: ObjectReference) {
        unimplemented!("OCaml 5.x finalizers — implement after basic GC works")
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
