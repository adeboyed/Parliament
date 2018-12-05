# Add repository for installing OCaml packages
add-apt-repository ppa:avsm/ppa
apt-get update

# Install OCaml packages & relevant dependencies
apt-get install ocaml ocaml-native-compilers camlp4-extra opam
apt-get install make
apt-get install libzmq3-dev libgmp3-dev m4 pkg-config

# Start opam
opam init

#Detext config
opam depext conf-pkg-config
