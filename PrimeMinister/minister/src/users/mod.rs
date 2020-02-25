/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::collections::HashSet;

pub mod handlers;
pub mod server;

pub struct User {
    pub id: String,
    pub last_request: u64,
    pub jobs: HashSet<String>,
    pub to_be_deleted: bool,
    pub docker_name: String,
}
