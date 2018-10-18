PROGRAM=Parliament

# OCaml build tool.
BUILDER=ocamlbuild

# OCaml libraries outside of the stdlib.
LIBS=owl

# $(DOCFILE).odocl must exist in $(SRCDIR) and 
# contain a list of module names (not file names) 
# to be documented.
DOCFILE=Parliament

# Where everything is stored
SRCDIR=src/main
BUILDDIR=_build

# Path separator for the current platform.
# Uncomment the next line for Windows platforms.
#/ := $(strip \)
# Uncomment the next line for UNIX platforms.
/=/

all: byte native
byte:
	$(BUILDER).byte $(SRCDIR)$/$(PROGRAM).cma -use-ocamlfind
native:
	$(BUILDER).native $(SRCDIR)$/$(PROGRAM).cmxa -use-ocamlfind
clean: 
	$(BUILDER) -clean -build-dir $(BUILDDIR)
