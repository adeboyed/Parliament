/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::net::TcpStream;
use std::sync::{Arc, RwLock};

use byteorder::{BigEndian, WriteBytesExt};
use crossbeam_channel::Sender;
use log::{error, info, warn};
use protobuf::{CodedOutputStream, Message};

use config::{Config, WorkerState, WorkerStatus};
use executor::{ServerMessage, TaskCommand};
use protos::intra_cluster::*;
use util;

pub trait RequestHandler {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        config: &Config,
        state: Arc<RwLock<WorkerState>>,
        executor_sender: &Sender<TaskCommand>,
        master_sender: &Sender<ServerMessage>,
    );
}

fn write_single_response(
    message_id: &String,
    single_response: SingleWorkerMessage,
    stream: &mut TcpStream,
) {
    let size = single_response.compute_size();
    if let Ok(_) = stream.write_u32::<BigEndian>(size) {
        let mut output_stream = CodedOutputStream::new(stream);

        match single_response.write_to(&mut output_stream) {
            Ok(_) => {
                (if let Err(e) = output_stream.flush() {
                    error!(
                        "{} || Could not flush stream! Error: {} ",
                        &message_id,
                        e.to_string()
                    );
                })
            }
            Err(e) => {
                error!(
                    "{} || Could not write to stream! Error: {}",
                    &message_id,
                    e.to_string()
                );
            }
        };
    } else {
        error!(
            "{} || Could not write the size of protobuf message to stream!",
            &message_id
        );
    }
}

fn send_heartbeat_message(
    worker_id: String,
    message_id: &String,
    state: Arc<RwLock<WorkerState>>,
    stream: &mut TcpStream,
) {
    let mut unwrapped_state = state.write().unwrap();

    if worker_id != unwrapped_state.worker_id {
        error!("{} || Worker ID is incorrect, ending!", &message_id);
        return; // Don't send back a response!
    }

    let mut heartbeat_response = WorkerHeartbeatResponse::new();
    unwrapped_state.last_request = util::current_secs();
    heartbeat_response.set_task_id(unwrapped_state.task_id.clone());
    match unwrapped_state.status {
        WorkerStatus::Awaiting => {
            heartbeat_response.set_status(WorkerHeartbeatResponse_HeartbeatStatus::AWAITING_TASK)
        }
        WorkerStatus::Finishing => {
            heartbeat_response.set_status(WorkerHeartbeatResponse_HeartbeatStatus::PROCESSING_TASK)
        }
        WorkerStatus::Processing => {
            heartbeat_response.set_status(WorkerHeartbeatResponse_HeartbeatStatus::PROCESSING_TASK)
        }
        WorkerStatus::Halted => {
            heartbeat_response.set_status(WorkerHeartbeatResponse_HeartbeatStatus::HALTED_TASK)
        }
        WorkerStatus::Disconnected => {
            warn!(
                "{} || Attempting to send a heartbeat request for a disconnected worker!",
                &message_id
            );
            return;
        }
    };

    info!(
        "{} || Worker ID: {} Sending state: {:#?}",
        &worker_id, &message_id, &heartbeat_response.status
    );

    let mut single_worker_message = SingleWorkerMessage::new();
    single_worker_message.set_heartbeat_response(heartbeat_response);
    write_single_response(message_id, single_worker_message, stream);
}

impl RequestHandler for WorkerHeartbeatRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        _config: &Config,
        state: Arc<RwLock<WorkerState>>,
        _executor_sender: &Sender<TaskCommand>,
        _master_sender: &Sender<ServerMessage>,
    ) {
        info!(
            "{} || Processing message as a WorkerHeartbeatRequest",
            &message_id
        );
        send_heartbeat_message(self.take_worker_id(), &message_id, state, stream);
    }
}

impl RequestHandler for WorkerTaskSubmissionRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        _config: &Config,
        state: Arc<RwLock<WorkerState>>,
        executor_sender: &Sender<TaskCommand>,
        _master_sender: &Sender<ServerMessage>,
    ) {
        info!(
            "{} || Processing message as a WorkerTaskSubmissionRequest",
            &message_id
        );
        let mut worker_id;
        {
            let mut worker_state = state.write().unwrap();
            worker_id = self.take_worker_id();

            if worker_id != worker_state.worker_id {
                error!("{} || Worker ID is incorrect, ending!", &message_id);
                return;
            }

            match worker_state.status {
                WorkerStatus::Awaiting => {
                    worker_state.data_in = Some(self.take_data_in());
                    worker_state.closure = Some(self.take_closure());
                    worker_state.task_id = self.take_task_id();
                    worker_state.task_type = Some(util::convert_map_type(&self.map_type));
                    match executor_sender.send(TaskCommand::StartTask(self.take_docker_name())) {
                        Ok(_) => (info!("{} || Accepted task successfully", &message_id)),
                        Err(e) => error!(
                            "{} || Could not put the StartTask command on the message bus! Err: {}",
                            &message_id,
                            e.to_string()
                        ),
                    }
                }
                WorkerStatus::Processing => {
                    warn!("{} || Attempting to submit task => We are still processing a job, cannot accept any new jobs at this time!", &message_id);
                }
                WorkerStatus::Finishing => {
                    warn!("{} || Worker is finishing, not accepting jobs", &message_id);
                }
                WorkerStatus::Halted => {
                    warn!(
                        "{} || Job errored, worker will be restarting soon!",
                        &message_id
                    );
                }
                WorkerStatus::Disconnected => {
                    warn!(
                        "{} || Worker is disconnected, how can we be receiving jobs!",
                        &message_id
                    );
                }
            };
        }
        send_heartbeat_message(worker_id, &message_id, state.clone(), stream);
    }
}

impl RequestHandler for WorkerTaskCancellationRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        config: &Config,
        state: Arc<RwLock<WorkerState>>,
        executor_sender: &Sender<TaskCommand>,
        master_sender: &Sender<ServerMessage>,
    ) {
        info!(
            "{} || Processing message as a WorkerTaskCancellationRequest",
            &message_id
        );

        match state.read().unwrap().status {
            WorkerStatus::Processing | WorkerStatus::Halted => {
                match executor_sender.send(TaskCommand::CancelTask) {
                    Ok(_) => (info!("{} || Cancelling task...", &message_id)),
                    Err(e) => error!(
                        "{} || Could not put the CancelJob command on the message bus! Err: {}",
                        &message_id,
                        e.to_string()
                    ),
                }
                send_heartbeat_message(self.take_worker_id(), &message_id, state.clone(), stream);
                return;
            }
            WorkerStatus::Awaiting => {
                warn!(
                    "{} || Attempting to cancel task => No job to cancel! In awaiting state",
                    &message_id
                );
            }
            WorkerStatus::Finishing => {
                warn!(
                    "{} || Attempting to cancel task => Worker is finishing, cannot cancel task",
                    &message_id
                );
            }
            WorkerStatus::Disconnected => {
                warn!(
                    "{} || Attempting to cancel task => Worker is disconnected, cannot cancel task",
                    &message_id
                );
            }
        };

        send_heartbeat_message(self.take_worker_id(), &message_id, state.clone(), stream);
        util::restart_worker(&config, &state, &master_sender, &executor_sender);
    }
}
