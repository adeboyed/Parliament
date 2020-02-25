/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::collections::HashSet;
use std::mem;
use std::sync::Arc;
use tokio::net::TcpStream;

use byteorder::{BigEndian, WriteBytesExt};
use chashmap::CHashMap;
use log::{debug, error, info, warn};
use protobuf::{CodedOutputStream, Message};

use crossbeam::queue::MsQueue;
use model::WJob;
use shared::protos::user_cluster::ServerMessage_Action::USER_TIMEOUT;
use shared::protos::user_cluster::*;
use users::User;
use util;

use protobuf::RepeatedField;

pub trait UserMessageHandler {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        users: Arc<CHashMap<String, User>>,
        jobs: Arc<CHashMap<String, WJob>>,
        jobs_queue: Arc<MsQueue<String>>,
        data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    );
}

fn write_single_response(
    message_id: &String,
    single_response: SingleUserResponse,
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

impl UserMessageHandler for CreateConnectionRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        users: Arc<CHashMap<String, User>>,
        _jobs: Arc<CHashMap<String, WJob>>,
        _jobs_queue: Arc<MsQueue<String>>,
        _data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    ) {
        info!(
            "{} || Processing message as a CreateConnectionRequest",
            &message_id
        );
        let mut return_message = CreateConnectionResponse::new();

        // -- ACCEPT USER --

        let mut user_id = self.take_authentication();

        if user_id.len() == 0 || !users.contains_key(&user_id) {
            if user_id.len() == 0 {
                user_id = util::unique_id(&users);
            }

            let docker_name = self.take_docker_name();
            info!(
                "{} || Accepting user! User ID: {}, Docker Name: {}",
                &message_id, &user_id, &docker_name
            );
            let new_user = User {
                id: user_id.clone(),
                last_request: util::current_secs(),
                jobs: HashSet::new(),
                to_be_deleted: false,
                docker_name,
            };
            users.insert(user_id.clone(), new_user);

            return_message.set_user_id(user_id);
            return_message.set_connection_accepted(true);
            info!("{} || Sending acceptance response back", &message_id);
        } else {
            error!("{} || User ID {} is not unique!", &message_id, &user_id);
            return_message.set_connection_accepted(false);
            info!("{} || Sending rejection response back", &message_id);
        }

        let mut single_response = SingleUserResponse::new();
        single_response.set_create_connection_response(return_message);
        return write_single_response(&message_id, single_response, stream);
    }
}

fn authenticate(
    message_id: &String,
    user_id: &String,
    users: &Arc<CHashMap<String, User>>,
) -> bool {
    match users.get_mut(user_id) {
        Some(mut user) => {
            user.last_request = util::current_secs();
            info!(
                "{} || Authenticated user with ID: {}",
                &message_id, &user_id
            );

            return !user.to_be_deleted;
        }
        None => {
            warn!(
                "{} || Attempted failed authentication from user {}",
                &message_id, &user_id
            );
            return false;
        }
    }
}

impl UserMessageHandler for ConnectionRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        users: Arc<CHashMap<String, User>>,
        _jobs: Arc<CHashMap<String, WJob>>,
        _jobs_queue: Arc<MsQueue<String>>,
        _data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    ) {
        info!(
            "{} || Processing message as a ConnectionRequest",
            &message_id
        );
        let mut single_response = SingleUserResponse::new();
        if authenticate(&message_id, &self.user_id, &users) {
            let mut connection_response = ConnectionResponse::new();
            match &self.action {
                ConnectionRequest_Action::HEARTBEAT => {
                    connection_response.set_request_accepted(true);
                }
                ConnectionRequest_Action::CLOSE_CONNECTION => {
                    if let Some(mut user) = users.get_mut(&mut self.user_id) {
                        user.to_be_deleted = true;
                        connection_response.set_request_accepted(true);
                    } else {
                        warn!(
                            "{}|| Could not find user, straight after authentication!",
                            &message_id
                        );
                        connection_response.set_request_accepted(false);
                    }
                }
            }
        } else {
            let mut return_message = ServerMessage::new();
            return_message.set_action(USER_TIMEOUT);
            single_response.set_server_message(return_message);
            warn!(
                "{} || Received request from unknown user with id {} ",
                &message_id, self.user_id
            );
        }

        return write_single_response(&message_id, single_response, stream);
    }
}

fn transfer_bytes(request: &mut InputAction, data_input: &mut Vec<Vec<u8>>) {
    let mut data_in = request.take_data_loc_in().to_vec();
    let mut i = 0;
    let len = data_in.len();

    while i < len {
        let data = mem::replace(&mut data_in[i], Vec::new());
        data_input.insert(i, data);
        i = i + 1;
    }
}

fn process_jobs(
    message_id: &String,
    docker_name: &String,
    input_jobs: RepeatedField<Job>,
    jobs: &Arc<CHashMap<String, WJob>>,
    user_id: &String,
    data: &Arc<CHashMap<String, Vec<Vec<u8>>>>,
) -> Result<Vec<WJob>, String> {
    let mut prev: String = "".parse().unwrap();
    let mut input_processed = false;

    let mut jobs_to_add: Vec<WJob> = Vec::new();

    let mut data_loc = "".parse().unwrap();
    let mut data_bytes: Vec<Vec<u8>> = Vec::new();

    if input_jobs.len() == 0 {
        warn!("{} || No jobs submitted!", &message_id);
        return Err("No jobs submitted".to_string());
    }

    info!("Processing {} jobs!", input_jobs.len());
    for j in input_jobs.into_iter() {
        if let Some(action) = j.action {
            match action {
                Job_oneof_action::map(mut map) => {
                    if input_processed {
                        let job_id = format!("{}-{}", user_id, j.job_id.to_string());

                        if jobs.contains_key(&job_id) {
                            error!(
                                "{} || Job {} already exists! Cancelling submission...",
                                &message_id, &job_id
                            );
                            return Err("There has been a job clash!".to_string());
                        }

                        let job = WJob::new(
                            job_id.clone(),
                            user_id.clone(),
                            prev,
                            docker_name.clone(),
                            util::convert_map_type(&map.mapType),
                            map.take_function_closure(),
                        );

                        // Change prev job next_job if exists
                        if !jobs_to_add.is_empty() {
                            let mut prev_job = jobs_to_add.pop().unwrap();
                            prev_job.set_output_id(job_id.clone());
                            jobs_to_add.push(prev_job);
                        }

                        prev = job_id;
                        jobs_to_add.push(job);
                    } else {
                        return Err(String::from(
                            "First job is a map job! Cancelling submission...",
                        ));
                    }
                }
                Job_oneof_action::input(mut input) => {
                    if input_processed {
                        return Err(String::from(
                            "Multiple input jobs! Cancelling submission...",
                        ));
                    } else {
                        let job_id = format!("{}-{}", user_id, j.job_id.to_string());

                        if jobs.contains_key(&job_id) {
                            error!(
                                "{} || Job {} already exists! Cancelling submission...",
                                &message_id, &job_id
                            );
                            return Err("There has been a job clash!".to_string());
                        }

                        prev = job_id.clone();
                        // LOAD IN DATA
                        data_loc = job_id;
                        transfer_bytes(&mut input, &mut data_bytes);

                        input_processed = true;
                    }
                }
            }
        } else {
            warn!(
                "{} || Job {} from user {} had no action! Ignoring...",
                &message_id, j.job_id, user_id
            );
            continue;
        }
    }

    debug!("Placing data at {}", &data_loc);
    data.insert(data_loc, data_bytes);
    return Ok(jobs_to_add);
}

impl UserMessageHandler for JobSubmission {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        users: Arc<CHashMap<String, User>>,
        jobs: Arc<CHashMap<String, WJob>>,
        jobs_queue: Arc<MsQueue<String>>,
        data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    ) {
        info!(
            "{} || Processing message as a JobSubmissionHandler",
            &message_id
        );
        let mut single_response = SingleUserResponse::new();

        if authenticate(&message_id, &self.user_id, &users) {
            let docker_name = users.get(&self.user_id).unwrap().docker_name.clone();
            match process_jobs(
                &message_id,
                &docker_name,
                self.take_jobs(),
                &jobs,
                &self.user_id,
                &data,
            ) {
                Ok(jobs_to_add) => {
                    info!(
                        "{} || Added jobs successfully! {}",
                        &message_id, &self.user_id
                    );

                    let first_job = jobs_to_add.first().unwrap().id.clone();
                    info!("{} || First job: {}", &message_id, &first_job);

                    for job in jobs_to_add {
                        info!("{} || Adding job {} ", &message_id, &job.id);
                        let mut user = users.get_mut(&mut self.user_id).unwrap();
                        user.jobs.insert(job.id.clone()); // Adding to user's job map
                        data.insert(job.id.clone(), Vec::new());
                        jobs.insert(job.id.clone(), job); // Adding to global static job map
                    }

                    jobs_queue.push(first_job);

                    let mut return_message = JobSubmissionResponse::new();
                    return_message.set_job_accepted(true);
                    single_response.set_job_submission_response(return_message);
                }
                Err(_) => {
                    warn!("{} || Could not add workload from user {}. Error encountered in validation/processing!", &message_id, self.user_id);

                    let mut return_message = JobSubmissionResponse::new();
                    return_message.set_job_accepted(false);
                    single_response.set_job_submission_response(return_message);
                }
            }
        } else {
            let mut return_message = ServerMessage::new();
            return_message.set_action(USER_TIMEOUT);
            single_response.set_server_message(return_message);
            warn!(
                "{} || Received request from unknown user with id {} ",
                &message_id, self.user_id
            );
        }

        return write_single_response(&message_id, single_response, stream);
    }
}

impl UserMessageHandler for DataRetrievalRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        users: Arc<CHashMap<String, User>>,
        jobs: Arc<CHashMap<String, WJob>>,
        _jobs_queue: Arc<MsQueue<String>>,
        data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    ) {
        info!(
            "{} || Processing message as a DataRetrievalRequest",
            &message_id
        );
        let mut single_response = SingleUserResponse::new();

        let mut error_message = ServerMessage::new();

        if authenticate(&message_id, &self.user_id, &users) {
            let user = users.get(&mut self.user_id).unwrap();
            let job_id = format!("{}-{}", self.user_id, self.job_id);
            if user.jobs.contains(&job_id) {
                if jobs.contains_key(&job_id) {
                    if data.contains_key(&job_id) {
                        let mut data_message = DataRetrievalResponse::new();
                        debug!(
                            "Data sent back size: {}",
                            (data.get(&job_id).unwrap()[0].len())
                        );
                        let data_bank = data.get(&job_id).unwrap();
                        data_message.set_bytes(RepeatedField::from_vec(data_bank.clone()));
                        single_response.set_data_retrieval_response(data_message);
                        return write_single_response(&message_id, single_response, stream);
                    } else {
                        warn!(
                            "{} || Could not find data for job {} !",
                            &message_id, self.job_id
                        );
                        error_message.set_action(ServerMessage_Action::INTERNAL_SERVER_ERROR);
                    }
                } else {
                    warn!("{} || Could not find job {} !", &message_id, self.job_id);
                    error_message.set_action(ServerMessage_Action::INTERNAL_SERVER_ERROR);
                }
            } else {
                warn!(
                    "{} || Could not find job {} for user {}!",
                    &message_id, self.job_id, self.user_id
                );
                error_message.set_action(ServerMessage_Action::MISSING_JOBS);
            }
        } else {
            warn!(
                "{} || Received request from unknown user with id {} ",
                &message_id, self.user_id
            );
            error_message.set_action(USER_TIMEOUT);
        }
        single_response.set_server_message(error_message);
        return write_single_response(&message_id, single_response, stream);
    }
}

impl UserMessageHandler for JobStatusRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        users: Arc<CHashMap<String, User>>,
        jobs: Arc<CHashMap<String, WJob>>,
        _jobs_queue: Arc<MsQueue<String>>,
        _data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    ) {
        info!(
            "{} || Processing message as a JobStatusRequest",
            &message_id
        );
        let mut single_response = SingleUserResponse::new();
        if authenticate(&message_id, &self.user_id, &users.clone()) {
            let mut status_response = JobStatusResponse::new();
            let mut statuses: RepeatedField<JobStatus> = RepeatedField::new();

            let user = users.get(&mut self.user_id).unwrap();
            for job_id in &self.job_ids {
                let unique_id = format!("{}-{}", self.user_id, job_id);

                if user.jobs.contains(&unique_id) {
                    let job = jobs.get(&unique_id).unwrap();

                    let mut status = JobStatus::new();
                    status.set_job_id(job_id.clone());
                    status.set_status(util::convert_job_status(&job.status));
                    debug!(
                        "{} || {} | {:?} ",
                        &message_id,
                        &job_id,
                        util::convert_job_status(&job.status)
                    );
                    statuses.push(status);
                } else {
                    warn!("{} || User {} tried to access an unknown job with {}. Sending response back now", &message_id, self.user_id, job_id);
                    let mut error_message = ServerMessage::new();
                    error_message.set_action(ServerMessage_Action::MISSING_JOBS);
                    single_response.set_server_message(error_message);
                    return write_single_response(&message_id, single_response, stream);
                }
            }
            status_response.set_job_statuses(statuses);
            single_response.set_job_status_response(status_response);
        } else {
            let mut return_message = ServerMessage::new();
            return_message.set_action(USER_TIMEOUT);
            single_response.set_server_message(return_message);
            warn!(
                "{} || Received request from unknown user with id {} ",
                &message_id, self.user_id
            );
        }

        return write_single_response(&message_id, single_response, stream);
    }
}
