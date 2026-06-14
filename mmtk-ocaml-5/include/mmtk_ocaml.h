#ifndef MMTK_OCAML_5_H
#define MMTK_OCAML_5_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque mutator handle; stored in caml_domain_state->mmtk_mutator. */
typedef void* MMTk_Mutator;

/* ── Initialisation ─────────────────────────────────────────────────── */

/**
 * Initialise MMTk.  Call once (from caml_startup) before any domain is created.
 *
 * @param heap_size  Total heap size in bytes.
 * @param plan       GC plan name: "NoGC", "MarkSweep", "Immix", "StickyImmix", …
 */
void mmtk_ocaml5_init(size_t heap_size, const char* plan);

/* ── Domain (mutator) lifecycle ─────────────────────────────────────── */

/**
 * Register an OCaml 5.x domain as an MMTk mutator.
 * Call from caml_domain_create after caml_domain_state is initialised.
 *
 * @param domain_state_addr  Address of the domain's caml_domain_state struct.
 * @return  Opaque mutator handle; store in caml_domain_state->mmtk_mutator.
 */
MMTk_Mutator mmtk_ocaml5_bind_mutator(uintptr_t domain_state_addr);

/**
 * Destroy the mutator for a terminating domain.
 * Call from caml_domain_stop.
 */
void mmtk_ocaml5_destroy_mutator(MMTk_Mutator mutator);

/* ── Allocation ─────────────────────────────────────────────────────── */

/**
 * Allocate an OCaml heap block.  Same semantics as the OCaml 4.14 version.
 * Returns a pointer to field 0.
 *
 * @param mutator   domain_state->mmtk_mutator.
 * @param wosize    Number of word-sized fields.
 * @param tag       OCaml block tag (0–255).
 * @param semantics 0 = default.
 */
void* mmtk_ocaml5_alloc(MMTk_Mutator mutator, size_t wosize, size_t tag, int semantics);

/* ── GC control ─────────────────────────────────────────────────────── */

void mmtk_ocaml5_handle_user_collection_request(uintptr_t domain_state_addr);

/* ── Safepoint protocol (called from patched caml_handle_gc_interrupt) ─ */

/**
 * True if a GC cycle is pending.  Polled from the domain interrupt handler.
 */
bool mmtk_ocaml5_wants_to_stop(void);

/**
 * Called by each domain when it reaches the GC interrupt safepoint.
 * Spins until the GC worker resumes all domains.
 *
 * @param num_domains  Total live domain count (for barrier coordination).
 */
void mmtk_ocaml5_safepoint_reached(size_t num_domains);

/* ── Object queries ─────────────────────────────────────────────────── */

bool mmtk_ocaml5_is_in_mmtk_spaces(const void* addr);

#ifdef __cplusplus
}
#endif

#endif /* MMTK_OCAML_5_H */
