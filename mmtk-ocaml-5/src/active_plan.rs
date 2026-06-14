//! VMActivePlan for OCaml 5.x — domain (mutator) registry.
//!
//! In OCaml 5.x the unit of parallelism is a *domain*, represented by
//! `caml_domain_state*`.  Each domain is one MMTk mutator.
//! A domain is registered on creation and deregistered on termination.
//!
//! Key difference from OCaml 4.14: domain count != thread count, and domains
//! can be created/destroyed dynamically.  The registry must be concurrent-safe
//! (RwLock or similar).

use std::collections::HashMap;
use std::sync::RwLock;

use lazy_static::lazy_static;

use mmtk::util::opaque_pointer::{VMMutatorThread, VMThread};
use mmtk::vm::ActivePlan;
use mmtk::Mutator;

use crate::OCaml5VM;

pub struct VMActivePlan;

/// Newtype wrapper around a raw mutator pointer so we can implement Send+Sync.
/// SAFETY: Access is always serialised by the surrounding RwLock.
struct MutatorPtr(*mut Mutator<OCaml5VM>);
unsafe impl Send for MutatorPtr {}
unsafe impl Sync for MutatorPtr {}

lazy_static! {
    /// Global domain registry: domain_state_addr → raw pointer to Mutator<OCaml5VM>.
    static ref DOMAIN_REGISTRY: RwLock<HashMap<usize, MutatorPtr>> =
        RwLock::new(HashMap::new());
}

/// Register a mutator for the given domain address.
pub fn register_mutator(domain_state_addr: usize, mutator: *mut Mutator<OCaml5VM>) {
    let mut map = DOMAIN_REGISTRY.write().unwrap();
    map.insert(domain_state_addr, MutatorPtr(mutator));
}

/// Deregister the mutator by pointer match and return it.
pub fn deregister_by_ptr(mutator_ptr: *mut Mutator<OCaml5VM>) {
    let mut map = DOMAIN_REGISTRY.write().unwrap();
    map.retain(|_k, v| v.0 != mutator_ptr);
}

impl ActivePlan<OCaml5VM> for VMActivePlan {
    /// True if `tls` corresponds to a registered OCaml 5.x domain.
    fn is_mutator(tls: VMThread) -> bool {
        let addr = tls.0.to_address().as_usize();
        let map = DOMAIN_REGISTRY.read().unwrap();
        map.contains_key(&addr)
    }

    /// Return the Mutator for the given domain.
    fn mutator(tls: VMMutatorThread) -> &'static mut Mutator<OCaml5VM> {
        let addr = tls.0 .0.to_address().as_usize();
        let map = DOMAIN_REGISTRY.read().unwrap();
        let ptr = map
            .get(&addr)
            .unwrap_or_else(|| panic!("OCaml 5.x mutator: unknown domain addr 0x{:x}", addr))
            .0;
        // SAFETY: MMTk guarantees this is only called during STW when the mutator is live.
        unsafe { &mut *ptr }
    }

    /// Iterator over all live domain mutators.
    fn mutators<'a>() -> Box<dyn Iterator<Item = &'a mut Mutator<OCaml5VM>> + 'a> {
        let map = DOMAIN_REGISTRY.read().unwrap();
        // Collect pointers under the lock, then drop the lock before iterating.
        let ptrs: Vec<*mut Mutator<OCaml5VM>> = map.values().map(|p| p.0).collect();
        drop(map);
        // SAFETY: Each pointer is live and MMTk calls this during STW only.
        let iter = ptrs.into_iter().map(|p| unsafe { &mut *p });
        Box::new(iter)
    }

    /// Count of currently registered domains.
    fn number_of_mutators() -> usize {
        DOMAIN_REGISTRY.read().unwrap().len()
    }
}
