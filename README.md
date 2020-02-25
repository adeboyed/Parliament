# Parliament

> **OCaml library for writing distributed computation**

### Installation

```bash
❯ make
❯ make install
```

### Regenerating Protobuf specification

1. Install [ocaml-protoc](https://github.com/mransan/ocaml-protoc)

```bash
❯ opam install ocaml-protoc
```

2. Run the protoc tool 

```bash
❯ ocaml-protoc -binary -ml_out ./ spec/protobuf/user-cluster/connection.proto
```

### How to develop for Parliament
See Parliament Example






