/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::sync::{Arc, RwLock};
use std::net::TcpStream;

use log::{info, error};
use protobuf::{CodedOutputStream, Message};
use byteorder::{WriteBytesExt, BigEndian};
use crossbeam_channel::Sender;

use config::{WorkerState, WorkerStatus};
use protos::user_cluster::*;

use executor::{ServerMessageType, TaskResult};
use util;


pub trait RequestHandler {
    fn handle_message(&mut self,
                      message_id: &String,
                      stream: &mut TcpStream,
                      state: Arc<RwLock<WorkerState>>,
                      master_sender: &Sender<::executor::ServerMessage>);
}


fn write_single_response(message_id: &String,
                         single_response: SingleWorkerResponse,
                         stream: &mut TcpStream) {

    let size = single_response.compute_size();
    if let Ok(_) = stream.write_u32::<BigEndian>(size) {
        let mut output_stream = CodedOutputStream::new(stream);

        match single_response.write_to(&mut output_stream) {
            Ok(_) => (
                if let Err(e) = output_stream.flush() {
                    error!("{} || Could not flush stream! Error: {} ", &message_id, e.to_string());
                }
            ),
            Err(e) => {
                error!("{} || Could not write to stream! Error: {}", &message_id, e.to_string());
            }
        };
    }else {
        error!("{} || Could not write the size of protobuf message to stream!", &message_id);
    }
}


impl RequestHandler for WorkerInputRequest {
    fn handle_message(&mut self,
                      message_id: &String,
                      stream: &mut TcpStream,
                      state: Arc<RwLock<WorkerState>>,
                      _master_sender: &Sender<::executor::ServerMessage>) {
        info!("{} || Processing message as a WorkerInputRequest", &message_id);

        let mut worker_state = state.read().unwrap().clone();

        let mut input_response = WorkerInputResponse::new();
        input_response.set_datapacks(worker_state.data_in.take().unwrap());
        input_response.set_map_type(util::convert_map_task_type(&worker_state.task_type.unwrap()));
        input_response.set_function_closure(worker_state.closure.take().unwrap());

        let mut single_response = SingleWorkerResponse::new();
        single_response.set_input_response(input_response);
        write_single_response(&message_id, single_response, stream);
    }
}

impl RequestHandler for WorkerOutputRequest {
    fn handle_message(&mut self,
                      message_id: &String,
                      stream: &mut TcpStream,
                      state: Arc<RwLock<WorkerState>>,
                      master_sender: &Sender<::executor::ServerMessage>) {
        info!("{} || Processing message as a WorkerOutputRequest", &message_id);

        state.write().unwrap().status = WorkerStatus::Finishing;
        master_sender.send(::executor::ServerMessage {
            message_type: ServerMessageType::FinishedRequest(TaskResult::JobFinished, Arc::new(self.take_datapacks())),
            retry_count: 0
        }).expect("Could not send finished request. Internal message broker is broken!");


        let mut single_response = SingleWorkerResponse::new();
        single_response.set_output_response(WorkerOutputResponse::new());
        write_single_response(&message_id, single_response, stream);
    }
}