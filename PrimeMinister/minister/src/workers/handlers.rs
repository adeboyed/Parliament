/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::sync::{Arc, RwLock};
use tokio::net::TcpStream;
use std::mem;
use std::collections::HashSet;

use log::{info, warn, error, trace};
use chashmap::CHashMap;
use protobuf::{CodedOutputStream, Message};
use byteorder::{WriteBytesExt, BigEndian};

use shared::protos::intra_cluster::{WorkerFinishedRequest, WorkerConnectionRequest, WorkerFinishedRequest_WorkerTaskStatus, ConsensusRequest};
use shared::protos::intra_cluster::{WorkerConnectionResponse, SingleServerMessage, WorkerFinishedResponse, ConsensusResponse, ConsensusRequest_Action};
use model::{Worker, WTask, TaskStatus};
use util;
use config::State;
use model::WorkerStatus;
use model::WorkerUpdate;
use crossbeam_channel::Sender;

pub trait RequestHandler {
    fn handle_message(&mut self,
                      message_id: &String,
                      stream: &mut TcpStream,
                      workers: &Arc<CHashMap<String, Worker>>,
                      tasks: &Arc<CHashMap<String, WTask>>,
                      worker_names: &Arc<RwLock<Vec<String>>>,
                      data: &Arc<CHashMap<String, Vec<Vec<u8>>>>,
                      consensus_mode: bool,
                      consensus_state: Arc<State>,
                      running_tasks: &Arc<RwLock<HashSet<String>>>,
                      update_sender: &Sender<WorkerUpdate>);
}

fn write_single_response(message_id: &String,
                         single_response: SingleServerMessage,
                         stream: &mut TcpStream) -> bool {
    let size = single_response.compute_size();
    if let Ok(_) = stream.write_u32::<BigEndian>(size.clone()) {
        let mut output_stream = CodedOutputStream::new(stream);
        match single_response.write_to(&mut output_stream) {
            Ok(_) => {
                if let Err(e) = output_stream.flush() {
                    error!("{} || Could not flush stream! Error: {} ", &message_id, e.to_string());
                    return false;
                }
            }
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


impl RequestHandler for WorkerConnectionRequest {
    fn handle_message(&mut self,
                      message_id: &String,
                      stream: &mut TcpStream,
                      workers: &Arc<CHashMap<String, Worker>>,
                      _tasks: &Arc<CHashMap<String, WTask>>,
                      worker_names: &Arc<RwLock<Vec<String>>>,
                      _data: &Arc<CHashMap<String, Vec<Vec<u8>>>>,
                      _consensus_mode: bool,
                      _consensus_state: Arc<State>,
                      _running_tasks: &Arc<RwLock<HashSet<String>>>,
                      _update_sender: &Sender<WorkerUpdate>) {
        info!("{} || Processing message as a WorkerConnectionRequest", &message_id);


        let mut single_response = SingleServerMessage::new();
        let mut connection_response = WorkerConnectionResponse::new();
        let mut id = self.take_authentication();

        let ip_addr = {
            let ip_override = self.take_ip_override();
            if ip_override.len() == 0 {
                stream.local_addr().unwrap().ip().to_string()
            } else {
                ip_override
            }
        };

        if id.len() == 0 || !workers.contains_key(&id) {
            if id.len() == 0 {
                id = util::unique_id(&workers);
            }

            worker_names.write().unwrap().push(id.clone());
            workers.insert(id.clone(), Worker::new(id.clone(), ip_addr, self.port.clone()));

            connection_response.set_worker_id(id);
            connection_response.set_connection_accepted(true);
        } else {
            error!("{} || Worker ID {} is not unique!", &message_id, &id);
            connection_response.set_connection_accepted(false);
            info!("{} || Sending rejection response back", &message_id);
        }

        single_response.set_connection_response(connection_response);
        write_single_response(&message_id, single_response, stream);
    }
}


fn transfer_bytes(request: &mut WorkerFinishedRequest,
                  data: &Arc<CHashMap<String, Vec<Vec<u8>>>>,
                  task: &WTask) {
    let mut output = data.get_mut(&task.data_out_id).unwrap();
    let mut data_out = request.take_data_out().to_vec();
    if task.data_out_loc == -1 {
        let mut i = 0;
        let len = data_out.len();

        while i < len {
            output.insert(i, mem::replace(&mut data_out[i], Vec::new()));
            i = i + 1;
        }
    } else {
        // Arguably we do not care what location it is in.
        output.push(mem::replace(&mut data_out[0], Vec::new()));
    }
}

impl RequestHandler for WorkerFinishedRequest {
    fn handle_message(&mut self,
                      message_id: &String,
                      stream: &mut TcpStream,
                      workers: &Arc<CHashMap<String, Worker>>,
                      tasks: &Arc<CHashMap<String, WTask>>,
                      worker_names: &Arc<RwLock<Vec<String>>>,
                      data: &Arc<CHashMap<String, Vec<Vec<u8>>>>,
                      consensus_mode: bool,
                      _consensus_state: Arc<State>,
                      running_tasks: &Arc<RwLock<HashSet<String>>>,
                      _update_sender: &Sender<WorkerUpdate>) {
        info!("{} || Processing message as a WorkerFinishedRequest", &message_id);

        let mut successful = false;

        match (workers.get(&self.worker_id), self.status) {
            (Some(worker), WorkerFinishedRequest_WorkerTaskStatus::TASK_ERRORED) => {
                info!("{} || WorkerFinishedRequest.status = TASK_ERRORED from {}", &message_id, &self.worker_id);
                if let Some(task_id) = worker.running_task.clone() {
                    if let Some(mut task) = tasks.get_mut(&task_id) {
                        task.status = TaskStatus::Halted;
                        successful = true;

                        info!("{} || Successfully processed WorkerFinishedRequest from worker {}", &message_id, &self.worker_id);
                    } else {
                        warn!("{} || Worker {} has given updates on task that does not exist anymore!", &message_id, &self.worker_id);
                    }
                } else if consensus_mode {
                    // We'll have to allow data to come in
                    let task_id = self.take_task_id();
                    if let Some(mut task) = tasks.get_mut(&task_id) {
                        task.status = TaskStatus::Halted;
                        successful = true;

                        info!("{} || Consensus allow, Successfully processed WorkerFinishedRequest from worker {}", &message_id, &self.worker_id);
                    } else {
                        warn!("{} || Consensus allow, Worker {} has given updates on task that does not exist anymore!", &message_id, &self.worker_id);
                    }
                } else {
                    warn!("{} || Worker {} has sent WorkerFinishedRequest when no record of job starting!", &message_id, &self.worker_id);
                }
            }
            (Some(worker), WorkerFinishedRequest_WorkerTaskStatus::TASK_FINISHED) => {
                info!("{} || WorkerFinishedRequest.status = TASK_FINISHED from {} ", &message_id, &self.worker_id);
                if let Some(task_id) = worker.running_task.clone() {
                    if let Some(mut task) = tasks.get_mut(&task_id) {
                        task.status = TaskStatus::Completed;
                        transfer_bytes(self, &data, &task);
                        successful = true;
                    } else {
                        warn!("{} || Worker {} has given updates on task that does not exist anymore: {}!", &message_id, &self.worker_id, &task_id);
                        successful = true;
                    }
                } else if consensus_mode {
                    // We'll have to allow data to come in
                    let task_id = self.take_task_id();
                    info!("Task ID received!: {}", &task_id);
                    if let Some(mut task) = tasks.get_mut(&task_id) {
                        info!("{} || Consensus allow, Successfully processed WorkerFinishedRequest from worker {}", &message_id, &self.worker_id);
                        running_tasks.write().unwrap().insert(task_id.clone());
                        task.status = TaskStatus::Completed;
                        transfer_bytes(self, &data, &task);
                        successful = true;
                    } else {
                        warn!("{} || Consensus allow, Worker {} has given updates on task that does not exist anymore!", &message_id, &self.worker_id);
                        successful = true;
                    }
                } else {
                    warn!("{} || Worker {} has sent WorkerFinishedRequest when no record of job starting!", &message_id, &self.worker_id);
                }
            }
            (None, _) => ()
        }
        let mut single_response = SingleServerMessage::new();
        let mut finished_response = WorkerFinishedResponse::new();
        finished_response.set_response_processed(successful);
        single_response.set_finished_response(finished_response);

        if write_single_response(&message_id, single_response, stream) && successful {
            let names_option = worker_names.write();
            if names_option.is_err() {
                error!("Cannot write lock on worker_names! Err: {}", names_option.unwrap_err());
            } else {
                util::vec_remove(&mut names_option.unwrap(), self.worker_id.clone());
            }
            workers.remove(&self.worker_id);
        }
    }
}


fn take_control(worker_names: &Arc<RwLock<Vec<String>>>,
                workers: &Arc<CHashMap<String, Worker>>,
                update_sender: &Sender<WorkerUpdate>) {
    trace!("Running take_control protocol!");
    let mut names = worker_names.write().unwrap();
    for worker_id in names.clone().into_iter() {
        let mut worker = workers.remove(&worker_id).unwrap();
        if worker.running_task.is_none() || worker.assigned {
            info!("Restarting worker {}", &worker_id);
            worker.status = WorkerStatus::Cancelled;
            update_sender.send(WorkerUpdate::cancellation(&worker));
            util::vec_remove(&mut names, worker_id);
        }else {
            info!("Keeping in {}, should be eligible for assignment!", &worker_id);
            workers.insert(worker_id.clone(), worker);
        }
    }
}

impl RequestHandler for ConsensusRequest {
    fn handle_message(&mut self,
                      message_id: &String,
                      stream: &mut TcpStream,
                      workers: &Arc<CHashMap<String, Worker>>,
                      _tasks: &Arc<CHashMap<String, WTask>>,
                      worker_names: &Arc<RwLock<Vec<String>>>,
                      _data: &Arc<CHashMap<String, Vec<Vec<u8>>>>,
                      _consensus_mode: bool,
                      consensus_state: Arc<State>,
                      _running_tasks: &Arc<RwLock<HashSet<String>>>,
                      update_sender: &Sender<WorkerUpdate>) {
        info!("{} || Processing message as a ConsensusRequest", &message_id);

        match self.action {
            ConsensusRequest_Action::SET_ACTIVE => {
                info!("{} || SETTING ACTIVE!", &message_id);
                consensus_state.active.write().unwrap().set_value(true);
                take_control(&worker_names, &workers, &update_sender);
            }
            ConsensusRequest_Action::SET_PASSIVE => {
                info!("{} || SETTING PASSIVE!", &message_id);
                consensus_state.active.write().unwrap().set_value(false);
            }
            ConsensusRequest_Action::SHUTDOWN => {
                info!("{} || SHUTDOWN", &message_id);
                std::process::exit(0);
            }
        }

        let mut single_response = SingleServerMessage::new();
        single_response.set_consensus_response(ConsensusResponse::new());
        write_single_response(&message_id, single_response, stream);
    }
}