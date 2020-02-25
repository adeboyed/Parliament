/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

extern crate protoc_rust;

use protoc_rust::Customize;
#[test]
#[ignore]
fn generate_proto() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/protos",
        input: &["protos/intra_cluster.proto", "protos/user_cluster.proto"],
        includes: &["protos"],
        customize: Customize {
            ..Default::default()
        },
    }).expect("protoc");
}