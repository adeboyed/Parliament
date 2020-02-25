/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::net::TcpStream;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::sync::{Arc, RwLock};
use std::io::Read;

use log::{trace, warn, error};
use crossbeam_channel::{Sender, Receiver};
use protobuf::{CodedOutputStream, Message, CodedInputStream, ProtobufResult, ProtobufError, RepeatedField};
use protobuf::error::WireError;
use byteorder::{WriteBytesExt, BigEndian, ReadBytesExt};

use config::{Config, WorkerState};
use executor::{ServerMessage, ServerMessageType, TaskResult};
use protos::intra_cluster::*;
use communication::response::ResponseHandler;
use util;
use executor::TaskCommand;


fn construct_message(message_id : &String, state: &Arc<RwLock<WorkerState>>, message: ServerMessageType) -> SingleWorkerMessage {
    return match message {
        ServerMessageType::ConnectionRequest(authentication, port) => {
            trace!("{} || Sending ConnectionRequest", &message_id);
            let mut connection_request = WorkerConnectionRequest::new();
            connection_request.authentication = authentication;
            connection_request.port = port;

            let mut single_request = SingleWorkerMessage::new();
            single_request.set_connection_request(connection_request);

            single_request
        },
        ServerMessageType::FinishedRequest(TaskResult::JobFinished, data) => {
            trace!("{} || Sending FinishedRequest::JobFinished", &message_id);
            let readable_state = state.read().unwrap();
            let mut finished_request = WorkerFinishedRequest::new();

            finished_request.set_worker_id(readable_state.worker_id.clone());
            finished_request.set_status(WorkerFinishedRequest_WorkerTaskStatus::TASK_FINISHED);
            finished_request.set_task_id(readable_state.task_id.clone());

            let data_out = RepeatedField::from_vec(data.to_vec());
            finished_request.set_data_out(data_out);


            let mut single_request = SingleWorkerMessage::new();
            single_request.set_finished_request(finished_request);

            single_request
        },
        ServerMessageType::FinishedRequest(TaskResult::JobErrored, _) => {
            trace!("{} || Sending FinishedRequest:JobErrored", &message_id);
            let readable_state = state.read().unwrap();
            let mut finished_request = WorkerFinishedRequest::new();

            finished_request.set_worker_id(readable_state.worker_id.clone());
            finished_request.set_status(WorkerFinishedRequest_WorkerTaskStatus::TASK_ERRORED);
            finished_request.set_task_id(readable_state.task_id.clone());

            let mut single_request = SingleWorkerMessage::new();
            single_request.set_finished_request(finished_request);

            single_request
        }
    };
}

fn write_single_response(message_id: &String,
                         message: SingleWorkerMessage,
                         stream: &mut TcpStream) -> bool {
    let size = message.compute_size();
    if let Ok(_) = stream.write_u32::<BigEndian>(size) {
        let mut output_stream = CodedOutputStream::new(stream);
        match message.write_to(&mut output_stream) {
            Ok(_) => (
                if let Err(e) = output_stream.flush() {
                    error!("{} || Could not flush stream! Error: {} ", &message_id, e.to_string());
                    return false;
                }
            ),
            Err(e) => {
                error!("{} || Could not write to stream! Error: {}", &message_id, e.to_string());
                return false;
            }
        };
    } else {
        error!("{} || Could not write the size of protobuf message to stream!", &message_id);
        return false;
    }
    return true;
}

fn process_input(mut stream: &TcpStream) -> ProtobufResult<SingleServerMessage> {
    let size_option = stream.read_u32::<BigEndian>();
    if size_option.is_err() {
        error!("Could not write to stream!");
        return Err(ProtobufError::WireError(WireError::Other));
    }
    let size = size_option.unwrap();
    let mut buffer = vec![0u8; size as usize];
    if let Ok(_) = stream.read_exact(&mut buffer) {
        let mut message = SingleServerMessage::new();
        let mut cis = CodedInputStream::from_bytes(&buffer);
        return match message.merge_from(&mut cis) {
            Ok(()) => Ok(message),
            Err(e) => Err(e)
        };
    } else {
        return Err(ProtobufError::WireError(WireError::Other));
    }
}

fn retry_message(mut message: ServerMessage,
                 message_id: &String,
                 master_sender: &Sender<ServerMessage>) {
    error!("{} || Failed to receive a correct response from the server. Retrying...", &message_id);

    if message.retry_count < 3 {
        error!("{} || Waiting 500ms before resending...", &message_id);
        thread::sleep(Duration::from_millis(500));

        message.retry_count = message.retry_count + 1;
        master_sender.send(message).expect("Internal message broker has crashed!")
    }else {
        error!("{} || Attempting to send a message 3 times! Quitting...", &message_id);
        std::process::abort();
    }
}

pub fn start(config: &Config,
             state: Arc<RwLock<WorkerState>>,
             master_sender: Sender<ServerMessage>,
             master_receiver: Receiver<ServerMessage>,
             executor_sender: Sender<TaskCommand>) ->  std::io::Result<JoinHandle<()>> {
    let master_host = config.master.hostname.clone();
    let master_port = config.master.port.clone();
    let cloned_config = config.clone();

    return thread::Builder::new().name("client".to_string()).spawn(move || {
        loop {
            let message = master_receiver.recv().expect("Internal message broker has crashed!");
            let message_id = util::random_alphanum_string(10);
            trace!("{} || Processing message to send to the server", &message_id);

            let stream_option = TcpStream::connect(format!("{}:{}", master_host, master_port));
            if stream_option.is_err() {
                error!("Could not connect to master Error: {}", stream_option.unwrap_err().to_string());
                retry_message(message, &message_id, &master_sender);
                continue;
            }

            let mut stream = stream_option.unwrap();
            let mut retry = true;
            if write_single_response(&message_id, construct_message(&message_id, &state, message.message_type.clone()), &mut stream) {
                match process_input(&stream) {
                    Ok(return_message) => {
                        if let Some(response) = return_message.message {
                            retry = !match response {
                                SingleServerMessage_oneof_message::connection_response(mut x) => x.handle_message( &cloned_config, &message_id, &state, &master_sender, &executor_sender),
                                SingleServerMessage_oneof_message::finished_response(mut x) => x.handle_message(&cloned_config, &message_id, &state, &master_sender, &executor_sender),
                                _ => {
                                    error!("{} || Received a request message on client port. Ignoring...", &message_id);
                                    false
                                }
                            };
                        } else {
                            warn!("{} || Message from server did not send an action.", &message_id);
                            retry = true;
                        }
                    }
                    Err(e) => {
                        error!("{} || Could not decode message from TCP stream Error: {}", &message_id, e.to_string())
                    }
                }
            } else {
                error!("{} || Failed to send message to server", &message_id);
            }

            if retry {
                retry_message(message, &message_id, &master_sender);
            }
        }
    });
}


