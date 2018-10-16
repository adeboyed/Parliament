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
SRCDIR=src
DOCDIR=docs
BUILDDIR=_build

# Path separator for the current platform.
# Uncomment the next line for Windows platforms.
#/ := $(strip \)
# Uncomment the next line for UNIX platforms.
/=/

# Symbolic links created by this Makefile (DO NOT EDIT).
SYMLINKS=$(PROGRAM) $(PROGRAM).byte $(DOCDIR)

all: byte native
docs:
	$(BUILDER) $(SRCDIR)$/$(DOCFILE).docdir/index.html -I $(SRCDIR) -build-dir $(BUILDDIR)
byte:
	$(BUILDER).byte $(SRCDIR)$/$(PROGRAM).byte -use-ocamlfind
native:
	$(BUILDER).native $(SRCDIR)$/$(PROGRAM).native -use-ocamlfind
clean: 
	$(BUILDER) -clean -build-dir $(BUILDDIR)
