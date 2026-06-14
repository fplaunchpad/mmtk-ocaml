//! VMCollection for OCaml 4.14 — Stop-The-World coordination.
//!
//! OCaml 4.14 STW mechanism:
//!   To stop all threads, set `caml_young_limit = caml_young_end` for every thread.
//!   Each thread's allocation fast-path checks young_ptr >= young_limit and calls
//!   `caml_gc_dispatch` (the slow path) when the limit is hit.  We install a hook
//!   there to call our poll_for_gc equivalent.
//!
//!   This module owns the two atomics (WANTS_TO_STOP, WORLD_HAS_STOPPED) and the
//!   extern "C" functions exported to the patched OCaml runtime.
//!
//! TODO (agent): implement stop_all_mutators, resume_mutators, spawn_gc_thread.
//! See docs/binding-design.md for the full protocol description.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use mmtk::vm::GCThreadContext;
use mmtk::util::opaque_pointer::{VMMutatorThread, VMThread, VMWorkerThread};
use mmtk::vm::Collection;
use mmtk::Mutator;

use crate::OCaml4VM;

pub struct VMCollection;

/// Set to true by the GC worker to request all mutators stop.
pub static WANTS_TO_STOP: AtomicBool = AtomicBool::new(false);

/// Set to true by the last mutator to reach a safepoint, signalling the GC worker.
pub static WORLD_HAS_STOPPED: AtomicBool = AtomicBool::new(false);

/// Count of mutators that have reached a safepoint and are spinning.
pub static NUM_STOPPED: AtomicUsize = AtomicUsize::new(0);

// ── C-exported safepoint helpers (called from patched caml_gc_dispatch) ──

/// Called by a mutator when it reaches a safepoint and WANTS_TO_STOP is set.
/// The last mutator to arrive sets WORLD_HAS_STOPPED.
#[no_mangle]
pub extern "C" fn mmtk_ocaml4_safepoint_reached(num_mutators: usize) {
    let stopped = NUM_STOPPED.fetch_add(1, Ordering::SeqCst) + 1;
    if stopped == num_mutators {
        WORLD_HAS_STOPPED.store(true, Ordering::SeqCst);
    }
    // Spin until the GC worker clears WANTS_TO_STOP.
    while WANTS_TO_STOP.load(Ordering::SeqCst) {
        std::hint::spin_loop();
    }
    NUM_STOPPED.fetch_sub(1, Ordering::SeqCst);
}

/// Returns true if a GC is pending — called from the patched caml_gc_dispatch.
#[no_mangle]
pub extern "C" fn mmtk_ocaml4_wants_to_stop() -> bool {
    WANTS_TO_STOP.load(Ordering::SeqCst)
}

// ── VMCollection impl ─────────────────────────────────────────────────────

impl Collection<OCaml4VM> for VMCollection {
    /// Flip caml_young_limit to trigger the slow path in all threads, then wait
    /// until every mutator has reached a safepoint.
    ///
    /// TODO: iterate all registered threads and set their caml_young_limit.
    /// Then spin on WORLD_HAS_STOPPED.  Call mutator_visitor for each mutator.
    fn stop_all_mutators<F>(_tls: VMWorkerThread, _mutator_visitor: F)
    where
        F: FnMut(&'static mut Mutator<OCaml4VM>),
    {
        todo!(
            "OCaml 4.14 stop_all_mutators: \
             set WANTS_TO_STOP=true, flip caml_young_limit on each thread, \
             spin until WORLD_HAS_STOPPED"
        )
    }

    /// Reset caml_young_limit for all threads and clear the stop flag.
    ///
    /// TODO: iterate all threads and restore their caml_young_limit.
    fn resume_mutators(_tls: VMWorkerThread) {
        todo!(
            "OCaml 4.14 resume_mutators: \
             set WANTS_TO_STOP=false, WORLD_HAS_STOPPED=false, \
             restore caml_young_limit for all threads"
        )
    }

    /// Mutators poll in the slow path (caml_gc_dispatch hook); no explicit block needed.
    fn block_for_gc(_tls: VMMutatorThread) {}

    /// Spawn a Rust thread running the MMTk GC worker loop.
    ///
    /// TODO: create a std::thread, call memory_manager::start_worker.
    fn spawn_gc_thread(_tls: VMThread, _ctx: GCThreadContext<OCaml4VM>) {
        todo!("OCaml 4.14 spawn_gc_thread: std::thread::spawn + start_worker")
    }
}
