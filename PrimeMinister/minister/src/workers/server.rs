/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]

    workers/server.rs - Server for communicating with the workers
*/

use futures::Future;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::future::*;
use tokio::prelude::*;

use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::sync::atomic::Ordering::SeqCst;
use std::thread::{Builder, JoinHandle};

use chashmap::CHashMap;
use crossbeam_channel::Sender;
use log::{error, info, trace, warn};

use config::{Server, State};
use model::{WTask, Worker, WorkerUpdate};
use shared::protos::intra_cluster::{SingleWorkerMessage, SingleWorkerMessage_oneof_message};
use shared::util;
use workers::handlers::RequestHandler;

fn server(
    listener: TcpListener,
    consensus_mode: bool,
    consensus_state: Arc<State>,
    worker_names: Arc<RwLock<Vec<String>>>,
    workers: Arc<CHashMap<String, Worker>>,
    tasks: Arc<CHashMap<String, WTask>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    running_tasks: Arc<RwLock<HashSet<String>>>,
    update_sender: Sender<WorkerUpdate>,
) {
    let server = listener
        .incoming()
        .map_err(|e| println!("error = {:?}", e))
        .for_each(move |stream| {
            let names = worker_names.clone();
            let workers = workers.clone();
            let data = data.clone();
            let tasks = tasks.clone();
            let consensus_mode = consensus_mode.clone();
            let consensus_state = consensus_state.clone();
            let running_tasks = running_tasks.clone();
            let update_sender = update_sender.clone();

            tokio::spawn({
                process_message(
                    stream,
                    names,
                    workers,
                    data,
                    tasks,
                    consensus_mode,
                    consensus_state,
                    running_tasks,
                    update_sender,
                );
                ok(())
            });
            Ok(())
        });

    tokio::run(server);
}

fn process_message(
    mut stream: TcpStream,
    names: Arc<RwLock<Vec<String>>>,
    workers: Arc<CHashMap<String, Worker>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    tasks: Arc<CHashMap<String, WTask>>,
    consensus_mode: bool,
    consensus_state: Arc<State>,
    running_tasks: Arc<RwLock<HashSet<String>>>,
    update_sender: Sender<WorkerUpdate>,
) -> impl Future<Item = (), Error = ()> + Send {
    let ip_addr = stream.local_addr().unwrap().ip().to_string();
    let message_id = util::random_alphanum_string(10);
    trace!(
        "{} || Message on worker port received @ ip_addr [{}] received: ",
        &message_id,
        &ip_addr
    );

    let mut message = SingleWorkerMessage::new();

    match util::process_input_non_blocking(&stream, &mut message, consensus_mode) {
        Ok(id) => {
            if id == 0 {
                handle_message(
                    &message_id,
                    message,
                    &mut stream,
                    names,
                    workers,
                    tasks,
                    data,
                    consensus_mode,
                    consensus_state,
                    running_tasks,
                    update_sender,
                );
            } else {
                let current_id = consensus_state.id_counter.load(SeqCst);
                if id > current_id {
                    //TODO consensus_state.id_counter.fetch_max(id, SeqCst);
                    handle_message(
                        &message_id,
                        message,
                        &mut stream,
                        names,
                        workers,
                        tasks,
                        data,
                        consensus_mode,
                        consensus_state,
                        running_tasks,
                        update_sender,
                    );
                } else {
                    error!(
                        "{} || DROPPED MESSAGE WITH ID: {} CURRENT COUNTER: {}",
                        &message_id, &id, &current_id
                    );
                }
            }
        }
        Err(e) => error!(
            "{} || Could not decode message from TCP stream, Error: {}",
            &message_id,
            e.to_string()
        ),
    }
    ok(())
}

fn handle_message(
    message_id: &String,
    worker_message: SingleWorkerMessage,
    stream: &mut TcpStream,
    worker_names: Arc<RwLock<Vec<String>>>,
    workers: Arc<CHashMap<String, Worker>>,
    tasks: Arc<CHashMap<String, WTask>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    consensus_mode: bool,
    consensus_state: Arc<State>,
    running_tasks: Arc<RwLock<HashSet<String>>>,
    update_sender: Sender<WorkerUpdate>,
) {
    if let Some(message) = worker_message.message {
        match message {
            SingleWorkerMessage_oneof_message::connection_request(mut x) => x.handle_message(
                &message_id,
                stream,
                &workers,
                &tasks,
                &worker_names,
                &data,
                consensus_mode,
                consensus_state,
                &running_tasks,
                &update_sender,
            ),
            SingleWorkerMessage_oneof_message::finished_request(mut x) => x.handle_message(
                &message_id,
                stream,
                &workers,
                &tasks,
                &worker_names,
                &data,
                consensus_mode,
                consensus_state,
                &running_tasks,
                &update_sender,
            ),
            SingleWorkerMessage_oneof_message::consensus_request(mut x) => x.handle_message(
                &message_id,
                stream,
                &workers,
                &tasks,
                &worker_names,
                &data,
                consensus_mode,
                consensus_state,
                &running_tasks,
                &update_sender,
            ),
            _ => {
                error!(
                    "{} || Received a message type that was not a request on the server port!",
                    &message_id
                );
            }
        };
    } else {
        warn!(
            "{} || Message from worker did not send an action.",
            &message_id
        );
    }
}

/*
    EXPORTED FUNCTIONS
*/

pub fn start(
    server_config: &Server,
    consensus_mode: bool,
    consensus_state: Arc<State>,
    worker_names: Arc<RwLock<Vec<String>>>,
    workers: Arc<CHashMap<String, Worker>>,
    tasks: Arc<CHashMap<String, WTask>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    running_tasks: Arc<RwLock<HashSet<String>>>,
    update_sender: Sender<WorkerUpdate>,
) -> std::io::Result<JoinHandle<()>> {
    info!(
        "Starting worker socket server, listening on port {}",
        &server_config.port
    );
    let sock_addr = format!("{}:{}", &server_config.ip, &server_config.port)
        .parse()
        .unwrap();
    return match TcpListener::bind(&sock_addr) {
        Ok(listener) => Builder::new().name("server".to_string()).spawn(move || {
            server(
                listener,
                consensus_mode,
                consensus_state,
                worker_names,
                workers,
                tasks,
                data,
                running_tasks,
                update_sender,
            )
        }),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    };
}
