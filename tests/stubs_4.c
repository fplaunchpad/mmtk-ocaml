#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <stdint.h>
#include <stdio.h>
#include <pthread.h>
#include "../mmtk-ocaml-4/include/mmtk_ocaml.h"

static MMTk_Mutator g_mutator = NULL;
static int g_init = 0;

static void ensure_init(void) {
    if (g_init) return;
    mmtk_init(256 * 1024 * 1024, "NoGC");
    uintptr_t tls = (uintptr_t)pthread_self();
    mmtk_initialize_collection(tls);
    g_mutator = mmtk_bind_mutator(tls);
    g_init = 1;
}

CAMLprim value caml_mmtk4_alloc_print(value wosize_v) {
    CAMLparam1(wosize_v);
    ensure_init();
    size_t wosize = (size_t)Long_val(wosize_v);
    void* ptr = mmtk_alloc(g_mutator, wosize, 0, 0);
    printf("[ocaml4] alloc(wosize=%zu) -> %p\n", wosize, ptr);
    fflush(stdout);
    CAMLreturn(Val_unit);
}
