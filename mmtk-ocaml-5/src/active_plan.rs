//! VMActivePlan for OCaml 5.x — domain (mutator) registry.
//!
//! In OCaml 5.x the unit of parallelism is a *domain*, represented by
//! `caml_domain_state*`.  Each domain is one MMTk mutator.
//! A domain is registered on creation and deregistered on termination.
//!
//! Key difference from OCaml 4.14: domain count != thread count, and domains
//! can be created/destroyed dynamically.  The registry must be concurrent-safe
//! (RwLock or similar).
//!
//! TODO (agent): implement the registry (lazy_static RwLock<HashMap>),
//! keyed by domain_state address, and wire it to the trait methods below.

use mmtk::util::opaque_pointer::{VMMutatorThread, VMThread};
use mmtk::vm::ActivePlan;
use mmtk::Mutator;

use crate::OCaml5VM;

pub struct VMActivePlan;

impl ActivePlan<OCaml5VM> for VMActivePlan {
    /// True if `tls` corresponds to a registered OCaml 5.x domain.
    fn is_mutator(_tls: VMThread) -> bool {
        todo!("OCaml 5.x is_mutator: check domain registry")
    }

    /// Return the Mutator for the given domain.
    fn mutator(_tls: VMMutatorThread) -> &'static mut Mutator<OCaml5VM> {
        todo!("OCaml 5.x mutator: look up domain_state in registry")
    }

    /// Iterator over all live domain mutators.
    fn mutators<'a>() -> Box<dyn Iterator<Item = &'a mut Mutator<OCaml5VM>> + 'a> {
        todo!("OCaml 5.x mutators: iterate domain registry")
    }

    /// Count of currently registered domains.
    fn number_of_mutators() -> usize {
        todo!("OCaml 5.x number_of_mutators: domain_registry.len()")
    }
}
