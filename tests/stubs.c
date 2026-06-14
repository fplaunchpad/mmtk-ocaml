#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <caml/fail.h>
#include <stdint.h>
#include <stdio.h>
#include <pthread.h>
#include "../mmtk-ocaml-5/include/mmtk_ocaml.h"

static MMTk_Mutator g_mutator = NULL;
static int g_init = 0;

/* TODO(thread-safety): g_init and g_mutator are not synchronized. This is safe
 * for single-threaded tests but would race under OCaml systhreads. A production
 * implementation must use pthread_once and keep a per-thread mutator handle. */
static void ensure_init(void) {
    if (g_init) return;
    mmtk_ocaml5_init(256 * 1024 * 1024, "NoGC");
    uintptr_t tls = (uintptr_t)pthread_self();
    mmtk_ocaml5_initialize_collection(tls);
    g_mutator = mmtk_ocaml5_bind_mutator(tls);
    g_init = 1;
}

CAMLprim value caml_mmtk5_alloc_print(value wosize_v) {
    CAMLparam1(wosize_v);
    ensure_init();
    size_t wosize = (size_t)Long_val(wosize_v);
    void* ptr = mmtk_ocaml5_alloc(g_mutator, wosize, 0, 0);
    if (!ptr) {
        caml_failwith("mmtk_ocaml5_alloc returned NULL");
    }
    printf("[ocaml] alloc(wosize=%zu) -> %p\n", wosize, ptr);
    fflush(stdout);
    CAMLreturn(Val_unit);
}
