/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::io::Read;
use std::io::{Error, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread::{self, Builder, JoinHandle};

use byteorder::{BigEndian, ReadBytesExt};
use log::{error, info, trace, warn};
use protobuf::error::WireError;
use protobuf::{CodedInputStream, Message, ProtobufError, ProtobufResult};

use consensus::request::ConsensusRequestHandler;
use shared::protos::consensus::*;
use shared::util;
use state::State;

/*
    EXPORTED FUNCTIONS
*/

fn server(listener: TcpListener, port: i32, state: Arc<RwLock<State>>) {
    info!(
        "Started consensus (internal) server, listening on port {}",
        port
    );
    for wrapped_stream in listener.incoming() {
        let cloned_state = state.clone();
        thread::spawn(move || {
            if let Ok(mut stream) = wrapped_stream {
                let ip_addr = stream.local_addr().unwrap().ip().to_string();
                let message_id = util::random_alphanum_string(10);
                trace!(
                    "{} || Message on consensus port received @ ip_addr [{}] received!",
                    &message_id,
                    ip_addr
                );
                match process_input(&stream) {
                    Ok(message) => {
                        handle_message(&message_id, message, &mut stream, cloned_state);
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

fn process_input(mut stream: &TcpStream) -> ProtobufResult<SingleConsensusRequest> {
    let size = stream.read_i32::<BigEndian>().unwrap();
    let mut buffer = vec![0u8; size as usize];
    if let Ok(_) = stream.read_exact(&mut buffer) {
        let mut message = SingleConsensusRequest::new();
        let mut cis = CodedInputStream::from_bytes(&buffer);
        return match message.merge_from(&mut cis) {
            Ok(()) => Ok(message),
            Err(e) => Err(e),
        };
    } else {
        return Err(ProtobufError::WireError(WireError::Other));
    }
}

fn handle_message(
    message_id: &String,
    worker_message: SingleConsensusRequest,
    stream: &mut TcpStream,
    state: Arc<RwLock<State>>,
) {
    if let Some(request) = worker_message.request {
        match request {
            SingleConsensusRequest_oneof_request::leader_connection_request(mut x) => {
                x.handle_message(&message_id, stream, state)
            }
            SingleConsensusRequest_oneof_request::heartbeat_request(mut x) => {
                x.handle_message(&message_id, stream, state)
            }
            SingleConsensusRequest_oneof_request::conflicting_action_request(mut x) => {
                x.handle_message(&message_id, stream, state)
            }
            SingleConsensusRequest_oneof_request::unique_id_request(mut x) => {
                x.handle_message(&message_id, stream, state)
            }
        };
    } else {
        warn!(
            "{} || Message from worker did not send an action.",
            &message_id
        );
    }
}

pub fn start(state: Arc<RwLock<State>>) -> std::io::Result<JoinHandle<()>> {
    let this = state.read().unwrap().this.clone();
    return match TcpListener::bind(format!("{}:{}", &this.ip, &this.con_port)) {
        Ok(listener) => {
            (Builder::new()
                .name("server".to_string())
                .spawn(move || server(listener, this.con_port, state)))
        }
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    };
}
