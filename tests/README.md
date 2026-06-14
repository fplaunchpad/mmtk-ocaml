# Tests — OCaml 5.x MMTk NoGC Binding

## Overview

These tests verify the NoGC MMTk binding for OCaml 5.x end-to-end: the pure-C harness
(`test_nogc_c.c`) exercises the C API directly — init, bind mutator, alloc, destroy — without
any OCaml runtime involvement; the OCaml program (`test_nogc.ml` + `stubs.c`) exercises the
same path through the real OCaml 5.x runtime, calling `mmtk_ocaml5_alloc` from a
`CAMLprim` stub linked into an `ocamlopt`-compiled binary. Together they confirm that the
`libmmtk_ocaml_5` cdylib initialises correctly, registers a mutator keyed on the domain
state address, and returns valid heap pointers for five consecutive allocations.

## Prerequisites

| Requirement | Notes |
|---|---|
| Rust (stable) | `rustup update stable` |
| OCaml 5.x | `opam switch fstar-fresh` (or any 5.x switch) |
| mmtk-core | checked out at `../../mmtk-core` relative to this repo root |
| gcc / pthread | standard system toolchain |

All commands below assume the worktree root as the working directory.

---

## Test A — Pure C harness (`test_nogc_c.c`)

**What it does.** Calls `mmtk_ocaml5_init` to create a 256 MB NoGC heap, passes
`pthread_self()` as the TLS handle to `mmtk_ocaml5_initialize_collection` and
`mmtk_ocaml5_bind_mutator`, then loops `wosize` 1–5 calling `mmtk_ocaml5_alloc` and
printing each returned pointer. Destroys the mutator on exit. No OCaml runtime is loaded.

**Build and run.**

```sh
cargo build -p mmtk-ocaml-5
gcc tests/test_nogc_c.c \
    -Immtk-ocaml-5/include \
    -L$(pwd)/target/debug -Wl,-rpath,$(pwd)/target/debug \
    -lmmtk_ocaml_5 -lpthread -ldl \
    -o tests/test_nogc_c_5
./tests/test_nogc_c_5
```

**Expected output.**

```
[c-test] Initializing MMTk OCaml5 NoGC...
[c-test] Mutator: 0x<addr>
[c-test] alloc(wosize=1) -> 0x<addr>
[c-test] alloc(wosize=2) -> 0x<addr>
[c-test] alloc(wosize=3) -> 0x<addr>
[c-test] alloc(wosize=4) -> 0x<addr>
[c-test] alloc(wosize=5) -> 0x<addr>
[c-test] PASSED
```

---

## Test B — OCaml 5.x program (`test_nogc.ml` + `stubs.c`)

**What it does.** `test_nogc.ml` calls an external stub `caml_mmtk5_alloc_print` (defined in
`stubs.c`) for `wosize` 1–5. On the first call, `stubs.c` lazily initialises MMTk (256 MB,
NoGC), binds a mutator keyed on `pthread_self()`, then allocates and prints the returned
pointer. The OCaml runtime is fully active: the binary is compiled with `ocamlopt` and the
stub is a proper `CAMLprim` that uses `CAMLparam`/`CAMLreturn`.

**Build and run.**

```sh
eval $(opam env --switch=fstar-fresh)
OCAML_INC=$(ocamlopt -where)
LIB_DIR=$(pwd)/target/debug
gcc -c tests/stubs.c -I"$OCAML_INC" -Immtk-ocaml-5/include -o tests/stubs.o
ocamlopt tests/stubs.o tests/test_nogc.ml \
    -ccopt "-L$LIB_DIR -Wl,-rpath,$LIB_DIR" \
    -cclib -lmmtk_ocaml_5 -cclib -lpthread \
    -o tests/test_nogc_5
./tests/test_nogc_5
```

**Expected output.**

```
MMTk OCaml5 NoGC test
[ocaml] alloc(wosize=1) -> 0x<addr>
[ocaml] alloc(wosize=2) -> 0x<addr>
[ocaml] alloc(wosize=3) -> 0x<addr>
[ocaml] alloc(wosize=4) -> 0x<addr>
[ocaml] alloc(wosize=5) -> 0x<addr>
All allocations OK
```

---

## What the addresses mean

Returned pointers (`0x200000xxxxx` range) are inside MMTk's managed heap, not the OCaml
domain's minor heap. Both tests use the `mmtk_ocaml5_*` API prefix throughout.

---

## Files

| File | Role |
|---|---|
| `test_nogc_c.c` | Pure-C end-to-end test; no OCaml runtime |
| `stubs.c` | `CAMLprim` C stub that bridges OCaml and MMTk; lazily inits MMTk |
| `test_nogc.ml` | OCaml 5.x test program; calls `caml_mmtk5_alloc_print` via external |

---

## Key difference from OCaml 4

The OCaml 5 binding keys the mutator to the domain state address (`caml_domain_state*`)
rather than a `pthread_t`, reflecting OCaml 5's multi-domain architecture; the
`active_plan.rs` domain registry (`RwLock<HashMap<usize, MutatorPtr>>`) tracks all live
mutators by that address.
