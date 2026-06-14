#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <pthread.h>
#include "../mmtk-ocaml-4/include/mmtk_ocaml.h"

int main(void) {
    printf("[test] Initializing MMTk NoGC with 256MB heap...\n");
    mmtk_init(256 * 1024 * 1024, "NoGC");

    uintptr_t tls = (uintptr_t)pthread_self();
    printf("[test] Starting GC workers...\n");
    mmtk_initialize_collection(tls);

    printf("[test] Binding mutator...\n");
    MMTk_Mutator mutator = mmtk_bind_mutator(tls);
    printf("[test] Mutator bound: %p\n", mutator);

    printf("[test] Allocating objects...\n");
    for (size_t wosize = 1; wosize <= 5; wosize++) {
        void* ptr = mmtk_alloc(mutator, wosize, 0, 0);
        if (ptr == NULL) {
            fprintf(stderr, "[test] FAILED: alloc returned NULL for wosize=%zu\n", wosize);
            return 1;
        }
        printf("[test] alloc(wosize=%zu) -> %p OK\n", wosize, ptr);
    }

    mmtk_destroy_mutator(mutator);
    printf("[test] PASSED\n");
    return 0;
}
