build:
	dune build @install

install: build
	dune install

doc:
	dune build @doc

run: build
	dune exec parliament

test:
	dune runtest

pin:
	opam pin add .

repin: build
	opam upgrade house

build-all:
	dune build --workspace jbuild-workspace.dev @install

test-all:
	dune build --workspace jbuild-workspace.dev @runtest

.PHONY: build test pin repin build-all test-all