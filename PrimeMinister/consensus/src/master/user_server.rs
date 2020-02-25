/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Error, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread::{self, Builder, JoinHandle};

use log::{error, info, trace, warn};
use protobuf::error::WireError;
use protobuf::{CodedInputStream, Message, ProtobufError, ProtobufResult};

use consensus::client::ConsensusUpdate;
use master::consensus;
use shared::protos::user_cluster::{SingleUserRequest, SingleUserRequest_oneof_request};
use shared::util::random_alphanum_string;
use state::State;
use tokio_threadpool::Sender;

fn process_input(mut stream: &TcpStream) -> ProtobufResult<SingleUserRequest> {
    let size = stream.read_i32::<BigEndian>().unwrap();
    let mut buffer = vec![0u8; size as usize];
    if let Ok(_) = stream.read_exact(&mut buffer) {
        let mut message = SingleUserRequest::new();
        let mut cis = CodedInputStream::from_bytes(&buffer);
        return match message.merge_from(&mut cis) {
            Ok(()) => Ok(message),
            Err(e) => Err(e),
        };
    } else {
        return Err(ProtobufError::WireError(WireError::Other));
    }
}

fn server(
    listener: TcpListener,
    port: i32,
    state: Arc<RwLock<State>>,
    threadpool_sender: Arc<Sender>,
    client_sender: crossbeam_channel::Sender<ConsensusUpdate>,
) {
    info!("Started user server, listening on port {}", port);
    for wrapped_stream in listener.incoming() {
        let state = state.clone();
        let threadpool_sender = threadpool_sender.clone();
        let client_sender = client_sender.clone();

        thread::spawn(move || {
            if let Ok(mut stream) = wrapped_stream {
                let ip_addr = stream.local_addr().unwrap().ip().to_string();
                let message_id = random_alphanum_string(10);
                trace!(
                    "{} || Message on user port received @ ip_addr [{}] received!",
                    &message_id,
                    ip_addr
                );
                match process_input(&stream) {
                    Ok(mut message) => {
                        handle_message(
                            &message_id,
                            message,
                            &mut stream,
                            state,
                            threadpool_sender,
                            client_sender,
                        );
                    }
                    Err(e) => error!(
                        "{} || Could not decode message from TCP stream Error: {}",
                        &message_id,
                        e.to_string()
                    ),
                }
            } else {
                warn!("Error in accepting an incoming stream!");
            }
        });
    }
}

fn handle_message(
    message_id: &String,
    user_message: SingleUserRequest,
    stream: &mut TcpStream,
    state: Arc<RwLock<State>>,
    threadpool_sender: Arc<Sender>,
    client_sender: crossbeam_channel::Sender<ConsensusUpdate>,
) {
    let mut cloned_message = user_message.clone();

    if user_message.request.is_some() {
        let get_order_id = || {
            if let Ok(conflicting_id) = consensus::get_conflicting_id(&message_id, &state) {
                Ok(conflicting_id)
            } else {
                error!(
                    "{} || Could not successfully get a conflict ID. Ending connection...",
                    &message_id
                );
                Err(())
            }
        };

        let get_unique_id = || {
            if let Ok(conflicting_id) = consensus::get_unique_id(&message_id, &state) {
                Ok(conflicting_id)
            } else {
                error!(
                    "{} || Could not successfully get a conflict ID. Ending connection...",
                    &message_id
                );
                Err(())
            }
        };

        let mut conflicting_id = 0;

        match &user_message.request {
            Some(SingleUserRequest_oneof_request::data_retrieval_request(_)) => {
                if let Ok(id) = get_order_id() {
                    info!("{} || Request type: data retrieval request", &message_id);
                    conflicting_id = id;
                } else {
                    return;
                }
            }
            Some(SingleUserRequest_oneof_request::job_status_request(_)) => {
                if let Ok(id) = get_order_id() {
                    info!("{} || Request type: job status request", &message_id);
                    conflicting_id = id;
                } else {
                    return;
                }
            }
            Some(SingleUserRequest_oneof_request::create_connection_request(_)) => {
                if let Ok(id) = get_unique_id() {
                    info!("{} || Request type: create connection request. Generated ID for new user: {}", &message_id, &id);
                    let mut connection_request = cloned_message.take_create_connection_request();
                    connection_request.set_authentication(id);
                    cloned_message.set_create_connection_request(connection_request);
                } else {
                    return;
                }
            }
            _ => {}
        };

        let bytes_to_write = {
            match consensus::send_user_request(
                &message_id,
                Arc::new(Box::new(cloned_message)),
                conflicting_id,
                &state,
                threadpool_sender,
                client_sender,
            ) {
                Ok(bytes) => bytes,
                Err(_) => {
                    error!(
                        "{} || Did not achieve consensus. Ending connection...",
                        &message_id
                    );
                    return;
                }
            }
        };

        stream.write_u32::<BigEndian>(bytes_to_write.len() as u32);
        stream.write_all(&bytes_to_write);
    } else {
        warn!(
            "{} || Message from worker did not send an action.",
            &message_id
        );
    }
}

pub fn start(
    state: Arc<RwLock<State>>,
    threadpool_sender: Arc<Sender>,
    client_sender: crossbeam_channel::Sender<ConsensusUpdate>,
) -> std::io::Result<JoinHandle<()>> {
    let move_state = state.clone();
    let readable_state = state.read().unwrap();
    let ip = readable_state.this.ip.clone();
    let port = readable_state.this.user_port.clone();

    return match TcpListener::bind(format!("{}:{}", &ip, &port)) {
        Ok(listener) => {
            (Builder::new().name("server".to_string()).spawn(move || {
                server(listener, port, move_state, threadpool_sender, client_sender)
            }))
        }
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    };
}
