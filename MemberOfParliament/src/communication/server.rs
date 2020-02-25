/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::io::Read;
use std::io::{Error, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};

use byteorder::{BigEndian, ReadBytesExt};
use crossbeam_channel::Sender;
use log::{error, info, trace, warn};
use protobuf::error::WireError;
use protobuf::{CodedInputStream, Message, ProtobufError, ProtobufResult};

use communication::request::RequestHandler;
use config::{Config, WorkerState};
use executor::{ServerMessage, TaskCommand};
use protos::intra_cluster::*;
use util;

fn server(
    listener: TcpListener,
    port: i32,
    config: Config,
    state: Arc<RwLock<WorkerState>>,
    executor_sender: Sender<TaskCommand>,
    master_sender: Sender<ServerMessage>,
) {
    info!(
        "Started worker socket server thread, listening on port {}",
        port
    );
    for wrapped_stream in listener.incoming() {
        let state = state.clone();
        let executor_sender = executor_sender.clone();
        let master_sender = master_sender.clone();
        let config = config.clone();
        thread::spawn(move || {
            if let Ok(mut stream) = wrapped_stream {
                let ip_addr = stream.local_addr().unwrap().ip().to_string();
                let message_id = util::random_alphanum_string(10);
                trace!(
                    "{} || Message on worker port received @ ip_addr [{}] received!",
                    &message_id,
                    ip_addr
                );
                match process_input(&stream) {
                    Ok(message) => {
                        if let Some(message) = message.message {
                            match message {
                                SingleServerMessage_oneof_message::heartbeat_request(mut x) => x
                                    .handle_message(
                                        &message_id,
                                        &mut stream,
                                        &config,
                                        state,
                                        &executor_sender,
                                        &master_sender,
                                    ),
                                SingleServerMessage_oneof_message::submission_request(mut x) => x
                                    .handle_message(
                                        &message_id,
                                        &mut stream,
                                        &config,
                                        state,
                                        &executor_sender,
                                        &master_sender,
                                    ),
                                SingleServerMessage_oneof_message::cancellation_request(mut x) => x
                                    .handle_message(
                                        &message_id,
                                        &mut stream,
                                        &config,
                                        state,
                                        &executor_sender,
                                        &master_sender,
                                    ),
                                _ => {
                                    error!("{} || Received a response message on server port. Ignoring...", &message_id);
                                }
                            }
                        } else {
                            warn!(
                                "{} || Message from server did not send an action.",
                                &message_id
                            );
                        }
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

fn process_input(mut stream: &TcpStream) -> ProtobufResult<SingleServerMessage> {
    let size = stream.read_u32::<BigEndian>().unwrap();
    let mut buffer = vec![0u8; size as usize];
    if let Ok(_) = stream.read_exact(&mut buffer) {
        let mut message = SingleServerMessage::new();
        let mut cis = CodedInputStream::from_bytes(&buffer);
        return match message.merge_from(&mut cis) {
            Ok(()) => Ok(message),
            Err(e) => Err(e),
        };
    } else {
        return Err(ProtobufError::WireError(WireError::Other));
    }
}

pub fn start(
    config: &Config,
    state: Arc<RwLock<WorkerState>>,
    executor_sender: Sender<TaskCommand>,
    master_sender: Sender<ServerMessage>,
) -> std::io::Result<JoinHandle<()>> {
    let export_ip = &config.worker.hostname;
    let port = config.worker.port.clone();

    let cloned_config = config.clone();

    return match TcpListener::bind(format!("{}:{}", export_ip, &port)) {
        Ok(listener) => {
            (thread::Builder::new()
                .name("server".to_string())
                .spawn(move || {
                    server(
                        listener,
                        port,
                        cloned_config,
                        state,
                        executor_sender,
                        master_sender,
                    )
                }))
        }
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    };
}
