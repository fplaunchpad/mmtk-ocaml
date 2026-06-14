# MMTk-OCaml Test Suites

End-to-end tests for each OCaml runtime version. Each subdirectory contains a
pure-C harness and an OCaml program that exercise the MMTk binding through the
C API and through the real OCaml runtime respectively.

| Directory | Runtime | Status |
|---|---|---|
| [`ocaml-4/`](ocaml-4/README.md) | OCaml 4.14 (single domain) | NoGC ✅ |
| [`ocaml-5/`](ocaml-5/README.md) | OCaml 5.x (domains) | NoGC ✅ |

All commands assume the repository root as the working directory.
