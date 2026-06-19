#include <stdio.h>
#include <stdint.h>
#include <pthread.h>
#include "../mmtk-ocaml-5/include/mmtk_ocaml.h"

int main(void) {
    printf("[c-test] Initializing MMTk OCaml5 NoGC...\n");
    mmtk_ocaml5_init(256 * 1024 * 1024, "NoGC");

    uintptr_t tls = (uintptr_t)pthread_self();
    mmtk_ocaml5_initialize_collection(tls);

    MMTk_Mutator mutator = mmtk_ocaml5_bind_mutator(tls);
    printf("[c-test] Mutator: %p\n", mutator);

    for (size_t wosize = 1; wosize <= 5; wosize++) {
        void* ptr = mmtk_ocaml5_alloc(mutator, wosize, 0, 0);
        if (!ptr) { fprintf(stderr, "FAILED: null ptr for wosize=%zu\n", wosize); return 1; }
        printf("[c-test] alloc(wosize=%zu) -> %p\n", wosize, ptr);
    }

    mmtk_ocaml5_destroy_mutator(mutator);
    printf("[c-test] PASSED\n");
    return 0;
}
