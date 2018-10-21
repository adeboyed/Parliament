#!/bin/bash
set -e

for i in "${@:2}"; do
    echo "ocaml-protoc -binary -ml_out $1 $i"
    ocaml-protoc -binary -ml_out $1 $i >/dev/null
done