set -e

OPAM_DEPENDS="ocamlfind ounit re"
OCAML_VERSION=4.07.0
OPAM_VERSION=2.0.0

echo "yes" | sudo add-apt-repository ppa:ocaml47+opam20
sudo apt-get update -qq
sudo apt-get install -qq ocaml ocaml-native-compilers camlp4-extra opam
export OPAMYES=1
export OPAMVERBOSE=1
echo OCaml version
ocaml -version
echo OPAM versions
opam --version
opam --git-version

opam init 
opam install ${OPAM_DEPENDS}
eval `opam config env`
make
# make test
