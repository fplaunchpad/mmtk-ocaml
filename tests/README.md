# OCaml 4.14 MMTk NoGC Test Suite

## Overview

These tests verify the NoGC MMTk binding end-to-end: that the Rust library
initialises MMTk, starts GC worker threads, binds a mutator, allocates objects,
and returns non-null pointers from the managed heap. Test A drives the C API
directly from a plain C harness with no OCaml runtime involved. Test B drives
the same API from a real OCaml 4.14 program via a thin C stub that bridges the
OCaml calling convention to the MMTk API, confirming that pointers pass
correctly through the C ABI in both directions.

## Prerequisites

- **Rust** stable toolchain (`rustup default stable`)
- **OCaml 4.14.2** via opam:
  ```
  opam switch create 4.14.2
  ```
- **mmtk-core** checked out at `../../mmtk-core` relative to this repository
  root

## Test A: Pure C harness (`test_nogc.c`)

`tests/test_nogc.c` calls the MMTk C API directly:

1. `mmtk_init(256 MB, "NoGC")` — initialises the heap with the NoGC plan.
2. `mmtk_initialize_collection(tls)` — starts GC worker threads.
3. `mmtk_bind_mutator(tls)` — binds the current thread as a mutator.
4. Loops `wosize = 1 .. 5`, calling `mmtk_alloc` for each size and printing the
   returned pointer. Exits with code 1 if any allocation returns NULL.
5. `mmtk_destroy_mutator` — releases the mutator before exit.

### Build and run

Run these commands from the worktree root:

```sh
cargo build -p mmtk-ocaml-4
gcc tests/test_nogc.c \
    target/debug/libmmtk_ocaml_4.a \
    -Immtk-ocaml-4/include -lpthread -ldl -lm \
    -o tests/test_nogc_4
./tests/test_nogc_4
```

### Expected output

```
[test] Initializing MMTk NoGC with 256MB heap...
[test] Starting GC workers...
[test] Binding mutator...
[test] Mutator bound: 0x<addr>
[test] Allocating objects...
[test] alloc(wosize=1) -> 0x<addr> OK
[test] alloc(wosize=2) -> 0x<addr> OK
[test] alloc(wosize=3) -> 0x<addr> OK
[test] alloc(wosize=4) -> 0x<addr> OK
[test] alloc(wosize=5) -> 0x<addr> OK
[test] PASSED
```

Rust's MMTk worker threads may also emit diagnostic lines to stderr before the
first `[test]` line.

## Test B: OCaml 4.14 program (`test_nogc.ml` + `stubs_4.c`)

`tests/stubs_4.c` exposes a single OCaml primitive
`caml_mmtk4_alloc_print(wosize_v)`. On first call it lazily calls
`mmtk_init`, `mmtk_initialize_collection`, and `mmtk_bind_mutator` to
initialise a global mutator, then calls `mmtk_alloc` and prints the returned
pointer with `printf`.

`tests/test_nogc.ml` declares the external binding, prints a header, calls
`mmtk_alloc_print` for `wosize = 1 .. 5`, then prints a footer.

### Build and run

```sh
eval $(opam env --switch=4.14.2)
OCAML_INC=$(ocamlopt -where)
LIB_DIR=$(pwd)/target/debug
gcc -c tests/stubs_4.c -I"$OCAML_INC" -Immtk-ocaml-4/include -o tests/stubs_4.o
ocamlopt tests/stubs_4.o tests/test_nogc.ml \
    -ccopt "-L$LIB_DIR -Wl,-rpath,$LIB_DIR" \
    -cclib -lmmtk_ocaml_4 -cclib -lpthread \
    -o tests/test_nogc_4_ml
./tests/test_nogc_4_ml
```

### Expected output

```
MMTk OCaml4 NoGC test
[ocaml4] alloc(wosize=1) -> 0x<addr>
[ocaml4] alloc(wosize=2) -> 0x<addr>
[ocaml4] alloc(wosize=3) -> 0x<addr>
[ocaml4] alloc(wosize=4) -> 0x<addr>
[ocaml4] alloc(wosize=5) -> 0x<addr>
All allocations OK
```

MMTk worker-thread diagnostics may appear on stderr interleaved with these
lines.

## What the addresses mean

The returned pointers (typically `0x200000xxxxx`) are inside MMTk's managed
heap region, not the OCaml minor or major heap. Each printed address is
evidence that MMTk allocated the object and returned a valid, non-null pointer
that survived the round-trip through the C ABI — from the OCaml bytecode
calling convention in `stubs_4.c`, through MMTk's Rust allocator, and back into
the OCaml `printf` call.

## Files

| File | Role |
|------|------|
| `tests/test_nogc.c` | Pure C harness — exercises the MMTk C API with no OCaml runtime |
| `tests/stubs_4.c` | C stub — bridges the OCaml 4.14 calling convention to the MMTk API |
| `tests/test_nogc.ml` | OCaml 4.14 driver — calls the stub for `wosize = 1..5` |
| `mmtk-ocaml-4/include/mmtk_ocaml.h` | C header for the MMTk binding API |
| `target/debug/libmmtk_ocaml_4.a` | Static library produced by `cargo build -p mmtk-ocaml-4` |
| `target/debug/libmmtk_ocaml_4.so` | Shared library used by the OCaml test binary |
