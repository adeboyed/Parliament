/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
    
    users/server.rs - Server for communicating with users
*/

use futures::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::future::*;
use tokio::prelude::*;

use std::io::{Error, ErrorKind};
use std::sync::atomic::Ordering::SeqCst;
use std::thread::{Builder, JoinHandle};

use chashmap::CHashMap;
use crossbeam::queue::MsQueue;
use log::{error, info, trace, warn};

use config::{Server, State};
use model::WJob;
use shared::protos::user_cluster::*;
use shared::util;
use users::handlers::*;
use users::User;

fn server(
    listener: TcpListener,
    consensus_mode: bool,
    consensus_state: Arc<State>,
    users: Arc<CHashMap<String, User>>,
    jobs: Arc<CHashMap<String, WJob>>,
    jobs_queue: Arc<MsQueue<String>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
) {
    let server = listener
        .incoming()
        .map_err(|e| println!("error = {:?}", e))
        .for_each(move |stream| {
            let users = users.clone();
            let jobs = jobs.clone();
            let jobs_queue = jobs_queue.clone();
            let data = data.clone();
            let consensus_mode = consensus_mode.clone();
            let consensus_state = consensus_state.clone();

            tokio::spawn({
                process_message(
                    stream,
                    users,
                    jobs,
                    jobs_queue,
                    data,
                    consensus_mode,
                    consensus_state,
                );
                ok(())
            });
            Ok(())
        });

    tokio::run(server);
}

fn process_message(
    mut stream: TcpStream,
    users: Arc<CHashMap<String, User>>,
    jobs: Arc<CHashMap<String, WJob>>,
    jobs_queue: Arc<MsQueue<String>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    consensus_mode: bool,
    consensus_state: Arc<State>,
) -> impl Future<Item = (), Error = ()> + Send {
    let ip_addr = stream.local_addr().unwrap().ip().to_string();
    let message_id = util::random_alphanum_string(10);
    trace!(
        "{} || Message on user port received @ ip_addr [{}] received!",
        &message_id,
        ip_addr
    );

    let mut message = SingleUserRequest::new();
    match util::process_input_non_blocking(&stream, &mut message, consensus_mode) {
        Ok(id) => {
            if id == 0 {
                handle_message(
                    &message_id,
                    message,
                    &mut stream,
                    users,
                    jobs,
                    jobs_queue,
                    data,
                );
            } else {
                let current_id = consensus_state.id_counter.load(SeqCst);
                if id > current_id {
                    //consensus_state.id_counter.fetch_max(id, SeqCst);
                    handle_message(
                        &message_id,
                        message,
                        &mut stream,
                        users,
                        jobs,
                        jobs_queue,
                        data,
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
    worker_message: SingleUserRequest,
    stream: &mut TcpStream,
    users: Arc<CHashMap<String, User>>,
    jobs: Arc<CHashMap<String, WJob>>,
    jobs_queue: Arc<MsQueue<String>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
) {
    if let Some(request) = worker_message.request {
        match request {
            SingleUserRequest_oneof_request::create_connection_request(mut x) => {
                x.handle_message(&message_id, stream, users, jobs, jobs_queue, data)
            }
            SingleUserRequest_oneof_request::connection_request(mut x) => {
                x.handle_message(&message_id, stream, users, jobs, jobs_queue, data)
            }
            SingleUserRequest_oneof_request::job_submission(mut x) => {
                x.handle_message(&message_id, stream, users, jobs, jobs_queue, data)
            }
            SingleUserRequest_oneof_request::data_retrieval_request(mut x) => {
                x.handle_message(&message_id, stream, users, jobs, jobs_queue, data)
            }
            SingleUserRequest_oneof_request::job_status_request(mut x) => {
                x.handle_message(&message_id, stream, users, jobs, jobs_queue, data)
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
    users: Arc<CHashMap<String, User>>,
    jobs: Arc<CHashMap<String, WJob>>,
    jobs_queue: Arc<MsQueue<String>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
) -> std::io::Result<JoinHandle<()>> {
    info!(
        "Starting user server, listening on port {}",
        &server_config.port
    );
    let sock_addr = format!("{}:{}", &server_config.ip, &server_config.port)
        .parse()
        .unwrap();
    return match TcpListener::bind(&sock_addr) {
        Ok(listener) => {
            (Builder::new().name("server".to_string()).spawn(move || {
                server(
                    listener,
                    consensus_mode,
                    consensus_state,
                    users,
                    jobs,
                    jobs_queue,
                    data,
                )
            }))
        }
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    };
}
