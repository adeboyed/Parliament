/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]

    util.rs - Helpful functions used everywhere
*/

use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use beanstalkd::Beanstalkd;
use crossbeam_channel::Sender;
use log::{info, warn};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use config::{Config, ConfigServer, WorkerState, WorkerStatus};
use executor::TaskCommand;
use executor::{ServerMessage, ServerMessageType, TaskType};
use protos::intra_cluster::WorkerTaskSubmissionRequest_MapType;
use protos::user_cluster::WorkerInputResponse_MapType;

pub fn restart_worker_no_send(
    config: &Config,
    state: &Arc<RwLock<WorkerState>>,
    master_sender: &Sender<ServerMessage>,
) {
    if config.single_run_mode {
        info!("ENDING PROCESS...");
        std::process::abort();
    } else {
        info!("RESTARTING...");

        let mut worker_state = state.write().unwrap();
        worker_state.task_type = None;
        worker_state.data_in = None;
        worker_state.closure = None;
        worker_state.status = WorkerStatus::Awaiting;

        // Place connection req in master
        master_sender
            .send(ServerMessage {
                message_type: ServerMessageType::ConnectionRequest(
                    "".to_string(),
                    config.worker.port.clone(),
                ),
                retry_count: 0,
            })
            .expect("Could not send connection request");
    }
}

pub fn split_and_validate_server(input: String) -> Option<ConfigServer> {
    let mut parts: Vec<&str> = input.split(":").collect();

    if parts.len() != 2 {
        warn!("{} does not have 2 parts!", &input);
        return None;
    }

    let port = parts.pop().unwrap();
    let hostname = parts.pop().unwrap();

    return Some(ConfigServer {
        hostname: hostname.to_string(),
        port: port.parse::<i32>().unwrap(),
    });
}

// Used for fault tolerance testing!
pub fn place_message_on_queue() {
    let mut beanstalkd = Beanstalkd::localhost().unwrap();
    let _ = beanstalkd.put("Message", 0, 0, 11300);
}

pub fn restart_worker(
    config: &Config,
    state: &Arc<RwLock<WorkerState>>,
    master_sender: &Sender<ServerMessage>,
    executor_sender: &Sender<TaskCommand>,
) {
    if config.single_run_mode {
        info!("ENDING PROCESS...");
        std::process::abort();
    } else {
        info!("RESTARTING...");

        let mut worker_state = state.write().unwrap();
        worker_state.task_type = None;
        worker_state.data_in = None;
        worker_state.closure = None;
        worker_state.status = WorkerStatus::Awaiting;

        executor_sender
            .send(TaskCommand::SetNone)
            .expect("Could not send task command request");

        // Place connection req in master
        master_sender
            .send(ServerMessage {
                message_type: ServerMessageType::ConnectionRequest(
                    "".to_string(),
                    config.worker.port.clone(),
                ),
                retry_count: 0,
            })
            .expect("Could not send connection request");
    }
}

pub fn int_to_bool(value: i32) -> bool {
    if value == 1 {
        return true;
    } else {
        return false;
    }
}

pub fn convert_map_type(map_type: &WorkerTaskSubmissionRequest_MapType) -> TaskType {
    return match map_type {
        WorkerTaskSubmissionRequest_MapType::SINGLE_IN_SINGLE_OUT => TaskType::SingleInSingleOut,
        WorkerTaskSubmissionRequest_MapType::MULTI_IN_SINGLE_OUT => TaskType::MultiInSingleOut,
        WorkerTaskSubmissionRequest_MapType::SINGLE_IN_MULTI_OUT => TaskType::SingleInMultiOut,
    };
}

pub fn convert_map_task_type(task_type: &TaskType) -> WorkerInputResponse_MapType {
    return match task_type {
        TaskType::SingleInSingleOut => WorkerInputResponse_MapType::SINGLE_IN_SINGLE_OUT,
        TaskType::MultiInSingleOut => WorkerInputResponse_MapType::MULTI_IN_SINGLE_OUT,
        TaskType::SingleInMultiOut => WorkerInputResponse_MapType::SINGLE_IN_MULTI_OUT,
    };
}

pub fn convert_status(status: &WorkerStatus) -> String {
    return match status {
        WorkerStatus::Disconnected => "Disconnected",
        WorkerStatus::Awaiting => "Awaiting",
        WorkerStatus::Halted => "Halted",
        WorkerStatus::Finishing => "Finishing",
        WorkerStatus::Processing => "Processing",
    }
    .to_string();
}

pub fn random_alphanum_string(length: usize) -> String {
    return thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .collect();
}

pub fn current_secs() -> u64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanity_int_to_bool() {
        assert_eq!(int_to_bool(1), true);
        assert_eq!(int_to_bool(0), false);
    }

    #[test]
    fn test_sanity_convert_map_type() {
        let type1 = WorkerTaskSubmissionRequest_MapType::SINGLE_IN_SINGLE_OUT;
        let type2 = WorkerTaskSubmissionRequest_MapType::MULTI_IN_SINGLE_OUT;
        let type3 = WorkerTaskSubmissionRequest_MapType::SINGLE_IN_MULTI_OUT;

        assert_eq!(convert_map_type(&type1), TaskType::SingleInSingleOut);
        assert_eq!(convert_map_type(&type2), TaskType::MultiInSingleOut);
        assert_eq!(convert_map_type(&type3), TaskType::SingleInMultiOut);
    }

    #[test]
    fn test_sanity_convert_task_type() {
        let type1 = TaskType::SingleInSingleOut;
        let type2 = TaskType::SingleInMultiOut;
        let type3 = TaskType::MultiInSingleOut;

        assert_eq!(
            convert_map_task_type(&type1),
            WorkerInputResponse_MapType::SINGLE_IN_SINGLE_OUT
        );
        assert_eq!(
            convert_map_task_type(&type2),
            WorkerInputResponse_MapType::SINGLE_IN_MULTI_OUT
        );
        assert_eq!(
            convert_map_task_type(&type3),
            WorkerInputResponse_MapType::MULTI_IN_SINGLE_OUT
        );
    }
}
