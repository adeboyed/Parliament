/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use futures::sync::oneshot;
use futures::{lazy, Future};
use std::collections::HashSet;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};
use std::thread::{Builder, JoinHandle};
use tokio_threadpool::ThreadPool;

use byteorder::{BigEndian, WriteBytesExt};
use chashmap::CHashMap;
use crossbeam_channel::{Receiver, Sender};
use log::{debug, error, info, warn};
use protobuf::{CodedOutputStream, Message, RepeatedField};

use config::State;
use crossbeam::queue::MsQueue;
use model::{TaskStatus, WTask, Worker, WorkerStatus, WorkerUpdate, WorkerUpdateType};
use shared::protos::intra_cluster::*;
use shared::util as sutil;
use std::thread;
use std::time;
use util;

fn create_server_message(
    message_id: &String,
    update: &WorkerUpdate,
    tasks: &Arc<CHashMap<String, WTask>>,
    data: &Arc<CHashMap<String, Vec<Vec<u8>>>>,
) -> SingleServerMessage {
    let mut single_server_message = SingleServerMessage::new();
    match &update.message {
        WorkerUpdateType::Heartbeat => {
            info!(
                "{} || Sending heartbeat message to worker {}",
                &message_id, &update.worker_id
            );
            let mut heartbeat_request = WorkerHeartbeatRequest::new();
            heartbeat_request.set_worker_id(update.worker_id.clone());
            single_server_message.set_heartbeat_request(heartbeat_request);
        }
        WorkerUpdateType::Cancellation => {
            info!(
                "{} || Sending cancellation message to worker {}",
                &message_id, &update.worker_id
            );
            let mut cancel_message = WorkerTaskCancellationRequest::new();
            cancel_message.set_worker_id(update.worker_id.clone());
            single_server_message.set_cancellation_request(cancel_message);
        }
        WorkerUpdateType::Submission(task_id) => {
            info!(
                "{} || Sending submission message to worker {}, assigning task: {}",
                &message_id, &update.worker_id, &task_id
            );
            let mut task = tasks.get_mut(&task_id).unwrap();

            task.status = TaskStatus::Running(update.worker_id.clone());

            let data_bank = data.get(&task.data_in_id).unwrap();
            let data = {
                if task.data_in_loc != -1 {
                    let mut single_vec = Vec::new();
                    single_vec.push(
                        data_bank
                            .get(task.data_in_loc.clone() as usize)
                            .unwrap()
                            .clone(),
                    );
                    RepeatedField::from_vec(single_vec)
                } else {
                    RepeatedField::from_vec(data_bank.clone())
                }
            };

            let closure = &task.closure;

            let mut submission_request = WorkerTaskSubmissionRequest::new();
            submission_request.set_worker_id(update.worker_id.clone());
            submission_request.set_task_id(task_id.clone());
            submission_request.set_data_in(data);
            submission_request.set_docker_name(task.docker_name.clone());
            submission_request.set_map_type(util::convert_map_task_type(&task.job_type));
            submission_request.set_closure(Vec::from(closure.as_slice()));

            single_server_message.set_submission_request(submission_request);
        }
    }
    return single_server_message;
}

fn increment_heartbeat(
    message_id: &String,
    workers: &Arc<CHashMap<String, Worker>>,
    worker_id: &String,
) {
    let worker_option = workers.get_mut(worker_id);
    if let Some(mut worker) = worker_option {
        worker.missed_heartbeats = worker.missed_heartbeats + 1;
        info!(
            "{} || Received no valid status from worker {}. Missed heartbeats: {}",
            &message_id, worker.id, worker.missed_heartbeats
        );
    } else {
        warn!(
            "{} || Attempted to update job, but could not find {} in the map!",
            &message_id, &worker_id
        );
    }
}

pub fn handle_heartbeat_response(
    message_id: &String,
    mut response: WorkerHeartbeatResponse,
    workers: &Arc<CHashMap<String, Worker>>,
    consensus_mode: &bool,
    consensus_state: &Arc<State>,
    tasks: &Arc<CHashMap<String, WTask>>,
    running_tasks: &Arc<RwLock<HashSet<String>>>,
    worker_id: String,
) {
    let worker_option = workers.get_mut(&worker_id);
    if let Some(mut worker) = worker_option {
        worker.status = util::convert_worker_status(&response.status);
        worker.missed_heartbeats = 0;
        info!(
            "{} || Received status [{:?}] from worker {}",
            &message_id, worker.status, worker.id
        );

        // Consensus passive mode
        if *consensus_mode && !consensus_state.active.read().unwrap().get_value() {
            info!("Passive mode!");
            if worker.status == WorkerStatus::Processing {
                worker.running_task = Some(response.task_id.clone());
                if let Some(mut task) = tasks.get_mut(&response.task_id) {
                    task.status = TaskStatus::Running(worker.id.clone());
                }
                info!("Added {} to running tasks", &response.task_id);
                running_tasks
                    .write()
                    .unwrap()
                    .insert(response.take_task_id());
            }
        }
    } else {
        warn!(
            "{} || Attempted to update job, but could not find {} in the map!",
            &message_id, &worker_id
        );
    }
}

fn process_message(
    mut update: WorkerUpdate,
    worker_names: Arc<RwLock<Vec<String>>>,
    workers: Arc<CHashMap<String, Worker>>,
    tasks: Arc<CHashMap<String, WTask>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    sender: Sender<WorkerUpdate>,
    receiver: Receiver<WorkerUpdate>,
    tasks_queue: Arc<MsQueue<String>>,
    consensus_mode: bool,
    consensus_state: Arc<State>,
    running_tasks: Arc<RwLock<HashSet<String>>>,
) {
    let message_id = sutil::random_alphanum_string(10);

    debug!(
        "Sending message to the client...| Message ID: {}",
        &message_id
    );
    let process_time = util::current_secs() - update.entrance_time;
    if process_time > 2 {
        warn!(
            "{} || Job took {}s in the queue!",
            &message_id, process_time
        );
    }

    let stream_res = TcpStream::connect(format!("{}:{}", update.ip_addr, update.ip_port));
    if stream_res.is_err() {
        error!(
            "{} || Could not connect to worker! Error: {}",
            &message_id,
            stream_res.unwrap_err().to_string()
        );
        increment_heartbeat(&message_id, &workers, &update.worker_id);
        return;
    }
    let mut stream = stream_res.unwrap();

    {
        let message = create_server_message(&message_id, &update, &tasks, &data);
        let size = message.compute_size(); // TODO Should error check!
        stream.write_u32::<BigEndian>(size.clone());
        let mut output_stream = CodedOutputStream::new(&mut stream);
        message.write_to(&mut output_stream).unwrap();
        output_stream.flush().unwrap();
    }

    let mut message = SingleWorkerMessage::new();
    match sutil::process_input(&mut stream, &mut message, false) {
        Ok(_) => {
            if let SingleWorkerMessage_oneof_message::heartbeat_response(response) =
                message.message.unwrap()
            {
                handle_heartbeat_response(
                    &message_id,
                    response,
                    &workers,
                    &consensus_mode,
                    &consensus_state,
                    &tasks,
                    &running_tasks,
                    update.worker_id.clone(),
                );
            } else {
                warn!(
                    "{} || Received a message from the server of an incorrect type!",
                    &message_id
                );
            }
        }
        Err(e) => {
            increment_heartbeat(&message_id, &workers, &update.worker_id);
            if update.retry_count.clone() > 0 {
                update.retry_count = update.retry_count - 1;
                info!("{} || Unsuccessful message, retrying...", &message_id);
                thread::sleep(time::Duration::from_millis(50));
                match sender.send(update) {
                    Ok(_) => (),
                    Err(e) => {
                        // TODO: Do something!
                        error!(
                            "{} || Could not add worker_update to channel! Error: {}",
                            &message_id,
                            e.to_string()
                        )
                    }
                }
            } else {
                match &update.message {
                    WorkerUpdateType::Heartbeat => (),
                    WorkerUpdateType::Cancellation => {
                        error!(
                            "{} || Task could not be cancelled, removing worker from pool...",
                            &message_id
                        );
                        let names_option = worker_names.write();
                        if names_option.is_err() {
                            error!(
                                "Cannot write lock on worker_names! Err: {}",
                                names_option.unwrap_err()
                            );
                        } else {
                            util::vec_remove(&mut names_option.unwrap(), update.worker_id.clone());
                        }
                        workers.remove(&update.worker_id);
                    }
                    WorkerUpdateType::Submission(task_id) => {
                        error!("{} || Task could not be assigned, unassigning task and removing worker from pool...", &message_id);
                        tasks_queue.push(task_id.clone());

                        let names_option = worker_names.write();
                        if names_option.is_err() {
                            error!(
                                "Cannot write lock on worker_names! Err: {}",
                                names_option.unwrap_err()
                            );
                        } else {
                            util::vec_remove(&mut names_option.unwrap(), update.worker_id.clone());
                        }
                        workers.remove(&update.worker_id);
                    }
                }
            }

            warn!(
                "{} || Could decode message from TCP stream, Error: {}",
                &message_id,
                e.to_string()
            )
        }
    }
}

pub fn start(
    threads: &i32,
    worker_names: Arc<RwLock<Vec<String>>>,
    workers: Arc<CHashMap<String, Worker>>,
    tasks: Arc<CHashMap<String, WTask>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    sender: Sender<WorkerUpdate>,
    receiver: Receiver<WorkerUpdate>,
    tasks_queue: Arc<MsQueue<String>>,
    consensus_mode: bool,
    consensus_state: Arc<State>,
    running_tasks: Arc<RwLock<HashSet<String>>>,
) -> Vec<JoinHandle<()>> {
    let mut client_threads = vec![];
    let pool = ThreadPool::new();
    info!("Starting {} worker transmission threads!", threads);

    loop {
        let mut update = receiver.recv().unwrap();
        let workers = workers.clone();
        let tasks = tasks.clone();
        let data = data.clone();
        let sender = sender.clone();
        let receiver = receiver.clone();
        let tasks_queue = tasks_queue.clone();
        let consensus_mode = consensus_mode.clone();
        let consensus_state = consensus_state.clone();
        let running_tasks = running_tasks.clone();
        let worker_names = worker_names.clone();
        pool.spawn(lazy(move || {
            process_message(
                update,
                worker_names,
                workers,
                tasks,
                data,
                sender,
                receiver,
                tasks_queue,
                consensus_mode,
                consensus_state,
                running_tasks,
            );
            Ok(())
        }));
    }
    return client_threads;
}
