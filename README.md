# mmtk-ocaml

MMTk memory management bindings for OCaml. This project implements the
[MMTk](https://www.mmtk.io) VMBinding trait for both OCaml 4.14 LTS and OCaml 5.x,
providing a drop-in GC backend for the OCaml runtime.

## Repository layout

```
mmtk-ocaml/
├── Cargo.toml              # Cargo workspace root
├── mmtk-ocaml-common/      # Shared: OCaml header format, FieldSlot, object scanning
├── mmtk-ocaml-4/           # OCaml 4.14 binding (single-domain, caml_young_limit STW)
│   ├── src/
│   ├── include/mmtk_ocaml.h
│   └── Cargo.toml
└── mmtk-ocaml-5/           # OCaml 5.x binding (multi-domain, interrupt_word STW)
    ├── src/
    ├── include/mmtk_ocaml.h
    └── Cargo.toml
```

`mmtk-ocaml-common` is a pure library crate — it has no C exports and is never
linked directly. Both version crates depend on it and re-export its types.

## Prerequisites

| Dependency | Version | Notes |
|---|---|---|
| Rust | stable 2021 | `rustup update stable` |
| mmtk-core | 0.32.0 | Clone to a sibling directory; path dep is `../../mmtk-core` |
| OCaml 4.14 | 4.14.2 | `opam switch create 4.14.2` |
| OCaml 5.x | 5.4.1 | `opam switch create 5.4.1` (or use existing) |

The workspace `Cargo.toml` expects `mmtk-core` at `../../mmtk-core` relative to
this directory. Adjust the path dep if your layout differs.

## Building

```bash
# Build both bindings
cargo build

# Build one binding only
cargo build -p mmtk-ocaml-4
cargo build -p mmtk-ocaml-5
```

Each binding produces:
- `target/debug/libmmtk_ocaml_4.a` / `.so` — OCaml 4.14 binding
- `target/debug/libmmtk_ocaml_5.a` / `.so` — OCaml 5.x binding

## Running tests

See `tests/README.md` on each branch for build and run instructions.

## Development workflow

OCaml 4 and OCaml 5 work is done in parallel git worktrees:

```bash
# OCaml 4 work
cd .claude/worktrees/ocaml-4   # branch: binding/ocaml-4

# OCaml 5 work
cd .claude/worktrees/ocaml-5   # branch: binding/ocaml-5
```

Each worktree has its own `target/` directory. Shared code in
`mmtk-ocaml-common` is read-only from each worktree's perspective — changes
there should be made on `main` and rebased into the version branches.

## Architecture

```
OCaml runtime (C)
       │  calls
       ▼
mmtk_alloc / mmtk_init / …   ← C API (include/mmtk_ocaml.h)
       │
       ▼
OCaml{4,5}VM : VMBinding      ← Rust, version-specific
       │  delegates
       ▼
mmtk-ocaml-common             ← header format, FieldSlot, scan_ocaml_object
       │
       ▼
mmtk-core (MMTk 0.32)         ← GC plans: NoGC → MarkSweep → Immix
```

The OCaml value representation uses LSB=1 for integer immediates. `FieldSlot`
in `mmtk-ocaml-common` filters these so MMTk never traces integers as heap
pointers.

## Status

| Component | OCaml 4.14 | OCaml 5.x |
|---|---|---|
| VMObjectModel | ✅ complete | ✅ complete |
| VMSlot (FieldSlot) | ✅ complete | ✅ complete |
| scan\_object | ✅ complete | ✅ complete |
| spawn\_gc\_thread | ✅ | ✅ |
| VMActivePlan | 🔧 sentinel only | ✅ domain registry |
| VMScanning (roots) | ⬜ todo | ⬜ todo |
| VMCollection (STW) | ⬜ todo | ⬜ todo |
| NoGC end-to-end | ✅ C + OCaml test | ✅ C + OCaml test |
| MarkSweep / Immix | ⬜ | ⬜ |
