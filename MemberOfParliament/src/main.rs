/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/
extern crate beanstalkd;
extern crate byteorder;
extern crate clap;
extern crate core;
extern crate crossbeam_channel;
extern crate log;
extern crate man;
extern crate protobuf;
extern crate rand;
extern crate shiplift;
extern crate simple_logger;
extern crate tokio;

use log::info;
use std::sync::{Arc, RwLock};

use clap::{clap_app, ArgMatches};
use crossbeam_channel::unbounded;

use communication::{client, server};
use config::{Config, WorkerStatus};
use man::prelude::*;

mod communication;
mod config;
mod executor;
mod protos;
mod util;

fn load_config(arg: ArgMatches) -> Config {
    if arg.is_present("MAN_PAGE") {
        let page = Manual::new("basic")
            .about("Worker implementation for a Parliament cluster")
            .author(Author::new("[NAME REDACTED]").email("[EMAIL REDACTED]"))
            .description("Member of Parliament is the worker node for the Parliament cluster. It exposes 2 TCP servers, one to talk to the Prime Minister node (master) and one for communicating with the process executing the task. Please ensure both ports are free or the process will fail to start.

            Standard tasks are run as a subprocess of the Member Of Parliament process and hence will run using the same permissions & any process constraints.

            To run Docker tasks, please ensure that the Docker daemon (https://docker.com) is installed on the machine and the running user has access to the docker group. Member of Parliament communicates using the default UNIX Docker socket (/var/run/docker.sock)")
            .option(
                Opt::new("config")
                    .short("-c")
                    .long("--config")
                    .help("Sets a custom config file"),
            )
            .option(
                Opt::new("single run mode")
                    .short("-s")
                    .long("--oneshot")
                    .help("Only processes one job before finishing"),
            )
            .option(
                Opt::new("timeout")
                    .short("-t")
                    .long("--timeout")
                    .help("No. of seconds to termination after no message received from master"),
            )
            .option(
                Opt::new("master server")
                    .long("--master")
                    .help("[IP:Port] of the user server of the Prime Minister. Default: 127.0.0.1:1240"),
            )
            .option(
                Opt::new("worker server")
                    .long("--worker")
                    .help("[IP:Port] of the exposed worker server, for communication with Prime Minister. Default: 0.0.0.0:1242"),
            )
            .option(
                Opt::new("executor server")
                    .long("--executor")
                    .help("[IP:Port] of the exposed worker server, for communication with Parliament processes. Default: 0.0.0.0:1100"),
            )
            .option(Opt::new("help").long("--help").help("View the help page"))
            .render();

        println!("{}", page);
        ::std::process::exit(0);
    } else {
        let mut config = Config::default();

        if let Some(single_run_mode) = arg.value_of("SINGLE_RUN_MODE") {
            config.single_run_mode =
                util::int_to_bool(single_run_mode.clone().parse::<i32>().unwrap());
        }

        if let Some(timeout) = arg.value_of("TIMEOUT") {
            config.timeout = timeout.clone().parse::<i32>().unwrap();
        }

        if let Some(worker_ip) = arg.value_of("WORKER_IP") {
            config.worker.hostname = worker_ip.to_string();
        }

        if let Some(worker_server) = arg.value_of("WORKER_SERVER") {
            if let Some(this) = util::split_and_validate_server(worker_server.to_string()) {
                config.worker = this;
            }
        }

        if let Some(master_server) = arg.value_of("MASTER_SERVER") {
            if let Some(this) = util::split_and_validate_server(master_server.to_string()) {
                config.master = this;
            }
        }

        if let Some(executor_server) = arg.value_of("EXECUTOR_SERVER") {
            if let Some(this) = util::split_and_validate_server(executor_server.to_string()) {
                config.executor = this;
            }
        }

        return config;
    }
}

fn print_config(config: &Config) {
    info!("Worker Config Set: ");
    info!("Single Run Mode: {}", &config.single_run_mode);
    info!("Timeout: {}", &config.timeout);
    info!("Export IP: {}", &config.worker.hostname);
    info!("Export Port: {}", &config.worker.port);
    info!("Executor IP: {}", &config.executor.hostname);
    info!("Executor Port: {}", &config.executor.port);
    info!("Master Hostname: {}", &config.master.hostname);
    info!("Master Port: {}", &config.master.port);
}

fn main() {
    //Setup logging
    simple_logger::init().unwrap();

    //Worker shared state
    let state = Arc::new(RwLock::new(config::WorkerState {
        status: WorkerStatus::Disconnected,
        worker_id: String::from(""),
        last_request: util::current_secs(),
        data_in: None,
        closure: None,
        task_type: None,
        task_id: "".to_string(),
    }));

    //PubSub for sending messages to master
    let (master_sender, master_receiver) = unbounded();
    let (executor_sender, executor_receiver) = unbounded();

    // Load in worker config
    let matches = clap_app!(member_of_parliament =>
        (version: "0.3")
        (author: "[NAME REDACTED] [EMAIL REDACTED]")
        (about: "Worker implementation for a Parliament cluster")

        (@arg CONFIG: -c --config +takes_value "Sets a custom config file")

        (@arg SINGLE_RUN_MODE: -s --oneshot +takes_value "Only processes one job before finishing")
        (@arg TIMEOUT: -t --timeout +takes_value "No. of seconds to termination after no message received from master")

        (@arg WORKER_SERVER: --worker +takes_value "[IP:Port] of the exposed worker server, for communication with Prime Minister. Default: 0.0.0.0:1242")

        (@arg EXECUTOR_SERVER: --executor +takes_value "[IP:Port] of the exposed worker server, for communication with Parliament processes. Default: 0.0.0.0:1100")

        (@arg MASTER_IP: --master +takes_value "[IP:Port] of the user server of the Prime Minister. Default: 127.0.0.1:1240")

        (@arg MAN_PAGE: -m --man "Display man page")
    );
    let config = load_config(matches.get_matches());
    print_config(&config);

    // Open a socket for the master to connect to
    server::start(
        &config,
        state.clone(),
        executor_sender.clone(),
        master_sender.clone(),
    )
    .expect("Could not start worker server!");

    //Start worker client to send information to the master
    client::start(
        &config,
        state.clone(),
        master_sender.clone(),
        master_receiver,
        executor_sender,
    )
    .expect("Could not start client!");

    executor::communication::server::start(&config, state.clone(), master_sender.clone())
        .expect("Could not start inter-worker communication port");

    executor::executor::start_executor(&config, state.clone(), master_sender, executor_receiver)
}
