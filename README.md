# mmtk-ocaml

MMTk memory management bindings for OCaml 5.x, implementing the
[MMTk](https://www.mmtk.io) VMBinding trait as a drop-in GC backend for the
OCaml runtime.

## Repository layout

```
mmtk-ocaml/
├── Cargo.toml              # Cargo workspace root
├── mmtk-core/              # mmtk-core (git submodule)
├── ocaml-5/                # OCaml 5.4.1 source (git submodule — built in-tree)
├── mmtk-ocaml-5/           # OCaml 5.x MMTk binding
│   ├── src/
│   ├── include/mmtk_ocaml.h
│   └── Cargo.toml
└── tests/                  # End-to-end tests (C + OCaml)
```

## Getting started

```bash
git clone --recursive https://github.com/your-org/mmtk-ocaml
cd mmtk-ocaml
```

If you already cloned without `--recursive`:

```bash
git submodule update --init
```

## Prerequisites

| Dependency | Version | Notes |
|---|---|---|
| Rust | stable 2021 | `rustup update stable` |
| GCC / make | system | for building OCaml and C tests |
| mmtk-core | pinned in submodule | no separate install needed |
| OCaml 5.4.1 | pinned in submodule | built in-tree via `make ocaml` |

## Building

```bash
# Build the Rust binding
cargo build

# Build the OCaml compiler from the submodule (needed for OCaml tests)
make ocaml
```

`make ocaml` runs `./configure && make` inside `ocaml-5/` and produces
`ocaml-5/ocamlopt.opt`. This is a one-time step; subsequent `make test` calls
skip it if the compiler is already built.

## Running tests

```bash
# C test only (no OCaml compiler needed)
make test-c

# Both tests (builds OCaml compiler first if not already done)
make test

# Release build
make test PROFILE=release
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
