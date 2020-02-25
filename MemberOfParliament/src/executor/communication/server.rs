/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::net::{TcpListener, TcpStream};
use std::io::{Error, ErrorKind};
use std::thread::{self, JoinHandle};
use std::io::Read;
use std::sync::{Arc, RwLock};

use log::{info, trace, warn, error};
use crossbeam_channel::Sender;
use protobuf::{CodedInputStream, ProtobufResult, Message, ProtobufError};
use protobuf::error::WireError;
use byteorder::{ReadBytesExt, BigEndian};

use config::{Config, WorkerState};
use protos::user_cluster::*;
use executor::communication::request::RequestHandler;
use util;
use std::time::{SystemTime, UNIX_EPOCH};


fn current_millis() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    return since_the_epoch.as_secs() * 1000 +
        since_the_epoch.subsec_nanos() as u64 / 1_000_000;
}

fn server(listener: TcpListener,
          port: i32,
          state: Arc<RwLock<WorkerState>>,
          master_sender: Sender<::executor::ServerMessage>) {
    info!("Started executor socket server thread, listening on port {}", port);
    for wrapped_stream in listener.incoming() {
        let state = state.clone();
        let channel = master_sender.clone();
        thread::spawn(move || {
            if let Ok(mut stream) = wrapped_stream {
                let ip_addr = stream.local_addr().unwrap().ip().to_string();
                let message_id = util::random_alphanum_string(10);
                trace!("{} || Message on executor port received @ ip_addr [{}] received!", &message_id, ip_addr);
                trace!("{} || Time: {}", &message_id, current_millis());
                match process_input(&stream) {
                    Ok(message) => {
                        handle_message(&message_id, message, &mut stream, state,  channel);
                    }
                    Err(e) => error!("{} || Could not decode message from TCP stream Error: {}", &message_id, e.to_string())
                }
            }else {
                warn!("Error in accepting an incoming stream!");
            }
        });
    }
}

fn process_input(mut stream: &TcpStream) -> ProtobufResult<SingleWorkerRequest> {
    if let Ok(size) = stream.read_u32::<BigEndian>() {
        let mut buffer = vec![0u8; size as usize];
        if let Ok(_) = stream.read_exact(&mut buffer) {
            let mut message = SingleWorkerRequest::new();
            let mut cis = CodedInputStream::from_bytes(&buffer);
            return match message.merge_from(&mut cis) {
                Ok(()) => Ok(message),
                Err(e) => Err(e)
            };
        }else {
            error!("Could not read actual message from stream!");
            return Err(ProtobufError::WireError(WireError::Other));
        }
    }else {
        error!("Could not read size from stream!");
        return Err(ProtobufError::WireError(WireError::Other));
    }
}

fn handle_message(message_id: &String,
                  worker_request: SingleWorkerRequest,
                  stream: &mut TcpStream,
                  state: Arc<RwLock<WorkerState>>,
                  master_sender: Sender<::executor::ServerMessage>) {

    if let Some(request) = worker_request.request {
        match request {
            SingleWorkerRequest_oneof_request::input_request(mut x) => x.handle_message(&message_id, stream, state, &master_sender),
            SingleWorkerRequest_oneof_request::output_request(mut x) => x.handle_message(&message_id, stream, state, &master_sender),
        }
    }else {
        warn!("{} || Message from executor did not send an action.", &message_id);
    }

}

pub fn start(config: &Config,
             state: Arc<RwLock<WorkerState>>,
             master_sender: Sender<::executor::ServerMessage>) -> std::io::Result<JoinHandle<()>> {
    let export_ip = &config.executor.hostname;
    let port = config.executor.port.clone();
    return match TcpListener::bind(format!("{}:{}", export_ip, &port)) {
        Ok(listener) => (thread::Builder::new().name("server".to_string()).spawn(move || server(listener, port, state, master_sender))),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string()))
    };
}