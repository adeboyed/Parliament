FROM ocaml/opam2:alpine-3.8
USER opam

##################### PREREQUISITES ########################

RUN sudo apk update
RUN sudo apk add m4 libressl-dev linux-headers openblas-dev
RUN opam update && opam switch create 4.03.0 && eval $(opam config env)
RUN opam install -y oasis dune ocaml-compiler-libs core lwt uucp uutf ocaml-protoc

ENV PATH /home/opam/.opam/4.03.0/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:$PATH

##################### INSTAL PARLIAMENT #####################

COPY Parliament Parliament
RUN sudo chown -R opam Parliament
RUN cd Parliament && \
    make install && \
    cd ..

##################### INSTALL PROJECT #####################

ADD . .
RUN sudo rm -rf Parliament
RUN sudo chown -R opam:nogroup . && \
    make build

FROM alpine
WORKDIR /app
COPY --from=0 /home/opam/opam-repository/_build/default/src/house.exe house.exe
COPY files files

ENTRYPOINT ["./house.exe"]