set -e

OPAM_DEPENDS="ocamlfind ounit"

echo "yes" | sudo add-apt-repository ppa:avsm/ocaml42+opam12
sudo apt-get update -qq
sudo apt-get install -qq ocaml-nox ocaml-native-compilers camlp4-extra opam libgsl0-dev libshp-dev libplplot-dev
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
