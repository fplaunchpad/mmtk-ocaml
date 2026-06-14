//! VMActivePlan for OCaml 4.14 — mutator (thread) registry.
//!
//! In OCaml 4.14 each OS thread that runs OCaml code is a mutator.
//! We maintain a global thread-keyed map: pthread_t → *mut Mutator<OCaml4VM>.
//!
//! Threads register themselves via mmtk_bind_mutator (api.rs) and deregister
//! via mmtk_destroy_mutator.
//!
//! TODO (agent): implement the registry (lazy_static RwLock<HashMap>),
//! and wire it to the trait methods below.

use mmtk::util::opaque_pointer::{VMMutatorThread, VMThread};
use mmtk::vm::ActivePlan;
use mmtk::Mutator;

use crate::OCaml4VM;

pub struct VMActivePlan;

impl ActivePlan<OCaml4VM> for VMActivePlan {
    /// True if `tls` is a registered OCaml 4.14 mutator thread.
    fn is_mutator(_tls: VMThread) -> bool {
        todo!("OCaml 4.14 is_mutator: check thread registry")
    }

    /// Return the Mutator for the given thread.
    ///
    /// # Safety
    /// MMTk guarantees this is only called while the mutator is live and STW is active.
    fn mutator(_tls: VMMutatorThread) -> &'static mut Mutator<OCaml4VM> {
        todo!("OCaml 4.14 mutator: look up thread in registry")
    }

    /// Iterator over all live mutators.  Called during STW to visit each mutator.
    fn mutators<'a>() -> Box<dyn Iterator<Item = &'a mut Mutator<OCaml4VM>> + 'a> {
        todo!("OCaml 4.14 mutators: iterate registry")
    }

    /// Count of currently registered mutator threads.
    fn number_of_mutators() -> usize {
        todo!("OCaml 4.14 number_of_mutators: registry.len()")
    }
}
