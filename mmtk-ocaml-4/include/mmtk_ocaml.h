#ifndef MMTK_OCAML_4_H
#define MMTK_OCAML_4_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle returned by mmtk_bind_mutator. Store in caml_domain_state. */
typedef void* MMTk_Mutator;

/* ── Initialisation ─────────────────────────────────────────────────── */

/**
 * Initialise MMTk.  Call once before any OCaml allocation.
 *
 * @param heap_size  Total heap size in bytes.
 * @param plan       GC plan name: "NoGC", "MarkSweep", "Immix", "StickyImmix", …
 */
void mmtk_init(size_t heap_size, const char* plan);

/* ── Mutator lifecycle ──────────────────────────────────────────────── */

/**
 * Register the calling OCaml thread as an MMTk mutator.
 * Call once per thread before the first allocation on that thread.
 *
 * @param tls  Platform thread ID cast to usize (e.g. (usize)pthread_self()).
 * @return     Opaque mutator handle; store in Thread_local_state or equivalent.
 */
MMTk_Mutator mmtk_bind_mutator(uintptr_t tls);

/**
 * Deregister and destroy the mutator for a terminating thread.
 * Call from caml_thread_stop or equivalent.
 */
void mmtk_destroy_mutator(MMTk_Mutator mutator);

/* ── Allocation ─────────────────────────────────────────────────────── */

/**
 * Allocate an OCaml heap block and write its header.
 *
 * Returns a pointer to field 0 (the standard OCaml object-reference convention).
 * The header word is written at result[-1].
 *
 * Replaces the caml_alloc_small / caml_alloc_shr fast path.
 *
 * @param mutator   Mutator handle from mmtk_bind_mutator.
 * @param wosize    Number of word-sized fields.
 * @param tag       OCaml block tag (0–255).
 * @param semantics 0 = default; non-zero values are MMTk AllocationSemantics.
 */
void* mmtk_alloc(MMTk_Mutator mutator, size_t wosize, size_t tag, int semantics);

/* ── GC control ─────────────────────────────────────────────────────── */

/** Request an immediate GC (equivalent to Gc.compact / Gc.full_major). */
void mmtk_handle_user_collection_request(uintptr_t tls);

/* ── Safepoint protocol (called from patched caml_gc_dispatch) ───────── */

/**
 * True if a GC cycle is pending.  Poll at every allocation and function-return
 * safepoint in the patched runtime.
 */
bool mmtk_ocaml4_wants_to_stop(void);

/**
 * Called by a mutator thread when it reaches a safepoint during a pending GC.
 * Spins until the GC worker calls resume_mutators.
 *
 * @param num_mutators  Total number of live mutator threads (for barrier coordination).
 */
void mmtk_ocaml4_safepoint_reached(size_t num_mutators);

/* ── Object queries ─────────────────────────────────────────────────── */

/** True if addr points into an MMTk-managed heap region. */
bool mmtk_is_in_mmtk_spaces(const void* addr);

#ifdef __cplusplus
}
#endif

#endif /* MMTK_OCAML_4_H */
