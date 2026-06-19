PROFILE  ?= debug

OCAML_SRC := ocaml-5
OCAMLOPT  := $(OCAML_SRC)/ocamlopt.opt

# === Rust binding ===

.PHONY: build
build:
	cargo build $(if $(filter release,$(PROFILE)),--release,)

# === OCaml runtime (built once from submodule) ===

.PHONY: ocaml
ocaml: $(OCAMLOPT)

$(OCAMLOPT): $(OCAML_SRC)/configure
	cd $(OCAML_SRC) && ./configure
	$(MAKE) -C $(OCAML_SRC) -j$(shell nproc)

# === Tests (delegated to tests/Makefile) ===

.PHONY: test-c
test-c: build
	$(MAKE) -C tests test-c PROFILE=$(PROFILE)

.PHONY: test-ocaml test
test-ocaml test: build ocaml
	$(MAKE) -C tests $@ PROFILE=$(PROFILE)

# === Housekeeping ===

.PHONY: clean clean-ocaml
clean:
	$(MAKE) -C tests clean

clean-ocaml:
	$(MAKE) -C $(OCAML_SRC) clean
