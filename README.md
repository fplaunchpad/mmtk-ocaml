# mmtk-ocaml

MMTk memory management bindings for OCaml 5.x, implementing the
[MMTk](https://www.mmtk.io) VMBinding trait as a drop-in GC backend for the
OCaml runtime.

## Repository layout

```
mmtk-ocaml/
├── Cargo.toml              # Cargo workspace root
├── mmtk-core/              # mmtk-core (git submodule — pinned commit)
├── mmtk-ocaml-5/           # OCaml 5.x binding (multi-domain, interrupt_word STW)
│   ├── src/
│   ├── include/mmtk_ocaml.h
│   └── Cargo.toml
└── tests/                  # End-to-end tests (C + OCaml)
```

## Getting started

```bash
git clone --recursive https://github.com/your-org/mmtk-ocaml
cd mmtk-ocaml
cargo build
```

If you already cloned without `--recursive`:

```bash
git submodule update --init
```

## Prerequisites

| Dependency | Version | Notes |
|---|---|---|
| Rust | stable 2021 | `rustup update stable` |
| mmtk-core | pinned in submodule | fetched automatically via `git submodule update --init` |
| OCaml 5.x | 5.4.0+ | system package or `opam switch create 5.4.0` |

## Building

```bash
cargo build
```

Produces `target/debug/libmmtk_ocaml_5.{a,so}`.

## Running tests

```bash
# C-only test (no OCaml runtime needed)
gcc -o tests/test_nogc_c tests/test_nogc_c.c \
    -I mmtk-ocaml-5/include -L target/debug -lmmtk_ocaml_5 -lpthread -ldl -lm
LD_LIBRARY_PATH=target/debug ./tests/test_nogc_c

# OCaml integration test (requires OCaml 5 ocamlopt)
gcc -c tests/stubs.c -I mmtk-ocaml-5/include -I $(ocamlopt -where) -o tests/stubs.o
ocamlopt tests/test_nogc.ml tests/stubs.o target/debug/libmmtk_ocaml_5.a \
    -cclib "-lpthread -ldl -lm" -o tests/test_nogc
LD_LIBRARY_PATH=target/debug ./tests/test_nogc
```

## Architecture

```
OCaml runtime (C)
       │  calls
       ▼
mmtk_alloc / mmtk_init / …   ← C API (include/mmtk_ocaml.h)
       │
       ▼
OCaml5VM : VMBinding          ← Rust (mmtk-ocaml-5)
       │  delegates
       ▼
mmtk-core                     ← GC plans: NoGC → MarkSweep → Immix
```

The OCaml value representation uses LSB=1 for integer immediates. `FieldSlot`
filters these so MMTk never traces integers as heap pointers.

## Status

| Component | OCaml 5.x |
|---|---|
| VMObjectModel | ✅ complete |
| VMSlot (FieldSlot) | ✅ complete |
| scan\_object | ✅ complete |
| spawn\_gc\_thread | ✅ |
| VMActivePlan | ✅ domain registry |
| VMScanning (roots) | ⬜ todo |
| VMCollection (STW) | ⬜ todo |
| NoGC end-to-end | ✅ C + OCaml test |
| MarkSweep / Immix | ⬜ |
