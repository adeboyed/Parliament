PROGRAM=Parliament

# OCaml build tool.
BUILDER=ocamlbuild

# Where everything is stored
SRCDIR=src/main
BUILDDIR=_build

# Where the protobuf files are stored 
PROTO_SRC_FILES=src/proto/*.proto
PROTO_DEST_DIR=src/main/Core/Protobuf

# Path separator for the current platform.
# Uncomment the next line for Windows platforms.
#/ := $(strip \)
# Uncomment the next line for UNIX platforms.
/=/

all: byte native
byte:
	scripts/generate_proto.sh $(PROTO_DEST_DIR) $(PROTO_SRC_FILES)
	$(BUILDER).byte $(SRCDIR)$/$(PROGRAM).cma -use-ocamlfind
native:
	scripts/generate_proto.sh $(PROTO_DEST_DIR) $(PROTO_SRC_FILES)
	$(BUILDER).native $(SRCDIR)$/$(PROGRAM).cmxa -use-ocamlfind
clean: 
	$(BUILDER) -clean -build-dir $(BUILDDIR)
