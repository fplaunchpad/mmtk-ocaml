# === Configuration (override on the command line) ===
# make PROFILE=release

PROFILE  ?= debug

CARGO_FLAGS := $(if $(filter release,$(PROFILE)),--release,)
LIBSTATIC   := target/$(PROFILE)/libmmtk_ocaml_5.a
INCLUDE     := mmtk-ocaml-5/include

OCAML_SRC   := ocaml-5
OCAMLOPT    := $(OCAML_SRC)/ocamlopt.opt
OCAML_WHERE := $(OCAML_SRC)/stdlib

# === Build ===

.PHONY: build
build:
	cargo build $(CARGO_FLAGS)

# === OCaml runtime (built from submodule) ===

.PHONY: ocaml
ocaml: $(OCAMLOPT)

$(OCAMLOPT): $(OCAML_SRC)/configure
	cd $(OCAML_SRC) && ./configure
	$(MAKE) -C $(OCAML_SRC) -j$(shell nproc)

# === Test binaries ===

tests/test_nogc_c: tests/test_nogc_c.c $(LIBSTATIC)
	gcc -o $@ $< -I $(INCLUDE) $(LIBSTATIC) -lpthread -ldl -lm

tests/stubs.o: tests/stubs.c $(OCAMLOPT)
	gcc -c -o $@ $< -I $(INCLUDE) -I $(OCAML_WHERE)

tests/test_nogc: tests/test_nogc.ml tests/stubs.o $(LIBSTATIC) $(OCAMLOPT)
	$(OCAMLOPT) $< tests/stubs.o $(LIBSTATIC) -cclib "-lpthread -ldl -lm" -o $@

# === Test targets ===

.PHONY: test test-c test-ocaml

test-c: tests/test_nogc_c
	./tests/test_nogc_c

test-ocaml: tests/test_nogc
	./tests/test_nogc

test: test-c test-ocaml

# === Housekeeping ===

.PHONY: clean clean-ocaml
clean:
	rm -f tests/test_nogc_c tests/test_nogc tests/stubs.o tests/*.cmi tests/*.cmx

clean-ocaml:
	$(MAKE) -C $(OCAML_SRC) clean
