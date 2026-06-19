PROFILE     ?= debug

LIBSTATIC   := ../target/$(PROFILE)/libmmtk_ocaml_5.a
INCLUDE     := ../mmtk-ocaml-5/include
OCAMLOPT    := ../ocaml-5/ocamlopt.opt
OCAML_WHERE := ../ocaml-5/runtime

# === Test binaries ===

test_nogc_c: test_nogc_c.c $(LIBSTATIC)
	gcc -o $@ $< -I $(INCLUDE) $(LIBSTATIC) -lpthread -ldl -lm

stubs.o: stubs.c $(OCAMLOPT)
	gcc -c -o $@ $< -I $(INCLUDE) -I $(OCAML_WHERE)

test_nogc: test_nogc.ml stubs.o $(LIBSTATIC) $(OCAMLOPT)
	OCAMLLIB=$(abspath $(dir $(OCAMLOPT))stdlib) \
	  $(OCAMLOPT) $< stubs.o $(LIBSTATIC) -cclib "-lpthread -ldl -lm" -o $@

# === Run targets ===

.PHONY: test-c
test-c: test_nogc_c
	./test_nogc_c

.PHONY: test-ocaml
test-ocaml: test_nogc
	./test_nogc

.PHONY: test
test: test-c test-ocaml

# === Housekeeping ===

.PHONY: clean
clean:
	rm -f test_nogc_c test_nogc stubs.o *.cmi *.cmx
