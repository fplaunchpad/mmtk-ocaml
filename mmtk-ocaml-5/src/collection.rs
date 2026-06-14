//! VMCollection for OCaml 5.x — Stop-The-World coordination.
//!
//! OCaml 5.x STW mechanism (domain.c):
//!   Each domain has an `interrupt_word` pointer.  To interrupt a domain, write
//!   a non-zero value to `*interrupt_word`; the domain's allocation fast-path
//!   checks this word and calls `caml_handle_gc_interrupt` when set.
//!   `caml_try_run_on_all_domains` / `caml_stop_the_world` in domain.c coordinate
//!   a global barrier: all domains call `caml_domain_stop_begin` when they pause
//!   and `caml_domain_stop_end` when they resume.
//!
//! The MMTk side mirrors this: stop_all_mutators requests the barrier and waits;
//! resume_mutators releases all waiting domains.
//!
//! TODO (agent): implement stop_all_mutators, resume_mutators, spawn_gc_thread.
//! See docs/binding-design.md for the protocol detail.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use mmtk::vm::GCThreadContext;
use mmtk::util::opaque_pointer::{VMMutatorThread, VMThread, VMWorkerThread};
use mmtk::vm::Collection;
use mmtk::Mutator;

use crate::OCaml5VM;

pub struct VMCollection;

/// Set to true when a GC cycle is requested.
pub static WANTS_TO_STOP: AtomicBool = AtomicBool::new(false);

/// Set to true once all domains have acknowledged the stop request.
pub static WORLD_HAS_STOPPED: AtomicBool = AtomicBool::new(false);

/// Number of domains that have reached their safepoint.
pub static NUM_STOPPED: AtomicUsize = AtomicUsize::new(0);

// ── C-exported safepoint helpers (called from patched caml_handle_gc_interrupt) ──

/// Called by each domain when it reaches its interrupt safepoint.
/// The last domain to arrive sets WORLD_HAS_STOPPED.
#[no_mangle]
pub extern "C" fn mmtk_ocaml5_safepoint_reached(num_domains: usize) {
    let stopped = NUM_STOPPED.fetch_add(1, Ordering::SeqCst) + 1;
    if stopped == num_domains {
        WORLD_HAS_STOPPED.store(true, Ordering::SeqCst);
    }
    while WANTS_TO_STOP.load(Ordering::SeqCst) {
        std::hint::spin_loop();
    }
    NUM_STOPPED.fetch_sub(1, Ordering::SeqCst);
}

/// Returns true if a GC is in progress — polled from patched domain interrupt handler.
#[no_mangle]
pub extern "C" fn mmtk_ocaml5_wants_to_stop() -> bool {
    WANTS_TO_STOP.load(Ordering::SeqCst)
}

// ── VMCollection impl ─────────────────────────────────────────────────────

impl Collection<OCaml5VM> for VMCollection {
    /// Write to each domain's interrupt_word, then wait for all domains to stop.
    ///
    /// TODO: iterate all registered domains, write to their interrupt_word,
    /// spin on WORLD_HAS_STOPPED, call mutator_visitor per domain.
    fn stop_all_mutators<F>(_tls: VMWorkerThread, _mutator_visitor: F)
    where
        F: FnMut(&'static mut Mutator<OCaml5VM>),
    {
        todo!(
            "OCaml 5.x stop_all_mutators: \
             set WANTS_TO_STOP=true, write to each domain's interrupt_word, \
             spin until WORLD_HAS_STOPPED"
        )
    }

    /// Clear the stop flag; all spinning domains will exit their safepoint spin.
    fn resume_mutators(_tls: VMWorkerThread) {
        todo!(
            "OCaml 5.x resume_mutators: \
             WANTS_TO_STOP=false, WORLD_HAS_STOPPED=false"
        )
    }

    /// Domains call caml_handle_gc_interrupt; no explicit block needed here.
    fn block_for_gc(_tls: VMMutatorThread) {}

    /// Spawn a Rust thread running the MMTk GC worker loop.
    fn spawn_gc_thread(_tls: VMThread, _ctx: GCThreadContext<OCaml5VM>) {
        todo!("OCaml 5.x spawn_gc_thread: std::thread::spawn + start_worker")
    }
}
