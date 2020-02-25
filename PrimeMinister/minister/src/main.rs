/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

#[macro_use]
extern crate lazy_static;

extern crate byteorder;
extern crate chashmap;
extern crate clap;
extern crate crossbeam;
extern crate crossbeam_channel;
extern crate futures;
extern crate log;
extern crate protobuf;
extern crate shared;
extern crate simple_logger;
extern crate tokio;
extern crate tokio_threadpool;

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use chashmap::CHashMap;
use clap::{clap_app, ArgMatches};
use crossbeam::queue::MsQueue;
use crossbeam_channel::unbounded;

use config::{Config, State};
use model::{WJob, WTask, Worker};
use users::User;

mod cluster;
mod config;
mod model;
mod users;
mod util;
mod workers;

lazy_static! {
    static ref USERS: Arc<CHashMap<String, User>> = Arc::new(CHashMap::new());
    static ref DATA: Arc<CHashMap<String, Vec<Vec<u8>>>> = Arc::new(CHashMap::new());
    static ref WORKER_NAMES: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
    static ref WORKERS: Arc<CHashMap<String, Worker>> = Arc::new(CHashMap::new());
    static ref JOBS: Arc<CHashMap<String, WJob>> = Arc::new(CHashMap::new());
    static ref JOBS_QUEUE: Arc<MsQueue<String>> = Arc::new(MsQueue::new());
    static ref TASKS: Arc<CHashMap<String, WTask>> = Arc::new(CHashMap::new());
    static ref TASK_QUEUE: Arc<MsQueue<String>> = Arc::new(MsQueue::new());
    static ref RUNNING_TASKS: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));
    static ref CONSENSUS_STATE: Arc<State> = Arc::new(State::default());
}

fn load_config(arg: ArgMatches) -> Config {
    let mut config = Config::default();

    if let Some(user_export) = arg.value_of("USER_EXPORT") {
        if let Some(this) = util::split_and_validate_server(user_export.to_string()) {
            config.user_server = this;
        }
    }
    if let Some(worker_export) = arg.value_of("WORKER_EXPORT") {
        if let Some(this) = util::split_and_validate_server(worker_export.to_string()) {
            config.worker_server = this;
        }
    }

    if let Some(threads) = arg.value_of("THREADS") {
        config.transmission_threads = threads.parse::<i32>().unwrap();
    }

    if arg.is_present("CONSENSUS") {
        config.consensus_mode = true;
    }

    return config;
}

fn main() {
    //Setup logging
    simple_logger::init().unwrap();

    let matches = clap_app!(myapp =>
        (version: "0.23")
        (author: "[Name REDACTED] [Email REDACTED]")
        (about: "Prime Minister")

        (@arg USER_EXPORT: --user -u +takes_value "[IP:Port] of the exposed user server. Default: 0.0.0.0:1241")
        (@arg WORKER_EXPORT: --worker -w +takes_value "[IP:Port] of the exposed worker server. Default: 0.0.0.0:1240")

        (@arg THREADS: --threads -t +takes_value "Number of transmission threads")
        (@arg CONSENSUS: --consensus -c "Use in consensus mode")
    );
    // Load in config
    let config = load_config(matches.get_matches());

    print_header();

    let (update_sender, update_receiver) = unbounded();

    users::server::start(
        &config.user_server,
        config.consensus_mode.clone(),
        CONSENSUS_STATE.clone(),
        USERS.clone(),
        JOBS.clone(),
        JOBS_QUEUE.clone(),
        DATA.clone(),
    )
    .expect("Could not start user server!");

    workers::server::start(
        &config.worker_server,
        config.consensus_mode.clone(),
        CONSENSUS_STATE.clone(),
        WORKER_NAMES.clone(),
        WORKERS.clone(),
        TASKS.clone(),
        DATA.clone(),
        RUNNING_TASKS.clone(),
        update_sender.clone(),
    )
    .expect("Could not start worker server!");

    let _worker_client = workers::client::start(
        &config.transmission_threads,
        WORKER_NAMES.clone(),
        WORKERS.clone(),
        TASKS.clone(),
        DATA.clone(),
        update_sender.clone(),
        update_receiver,
        TASK_QUEUE.clone(),
        config.consensus_mode.clone(),
        CONSENSUS_STATE.clone(),
        RUNNING_TASKS.clone(),
    );

    cluster::run(
        USERS.clone(),
        DATA.clone(),
        JOBS.clone(),
        JOBS_QUEUE.clone(),
        WORKER_NAMES.clone(),
        WORKERS.clone(),
        TASKS.clone(),
        TASK_QUEUE.clone(),
        RUNNING_TASKS.clone(),
        update_sender,
        config.consensus_mode.clone(),
        CONSENSUS_STATE.clone(),
    );
}

fn print_header() {
    println!();
    println!("Parliament - A distributed general-purpose cluster-computing framework for OCaml");
    println!("Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]");
    println!();
    println!();
}
