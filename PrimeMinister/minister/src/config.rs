/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use shared::BoolWrapper;
use std::sync::atomic::AtomicUsize;
use std::sync::RwLock;

#[derive(Clone, Debug)]
pub struct Server {
    pub ip: String,
    pub port: i32,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub worker_server: Server,
    pub user_server: Server,
    pub transmission_threads: i32,
    pub consensus_mode: bool,
}

#[derive(Debug)]
pub struct State {
    pub id_counter: AtomicUsize,
    pub active: RwLock<BoolWrapper>,
}

impl Default for Config {
    fn default() -> Config {
        return Config {
            worker_server: Server {
                ip: "0.0.0.0".to_string(),
                port: 1240,
            },
            user_server: Server {
                ip: "0.0.0.0".to_string(),
                port: 1241,
            },
            transmission_threads: 5,
            consensus_mode: false,
        };
    }
}

impl Default for State {
    fn default() -> State {
        return State {
            id_counter: AtomicUsize::new(0),
            active: RwLock::new(BoolWrapper::new(false)),
        };
    }
}
