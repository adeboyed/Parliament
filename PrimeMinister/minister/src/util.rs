/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]

    util.rs - Helpful functions used everywhere
*/

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chashmap::CHashMap;
use log::warn;

use config::Server;
use model::{JobStatus, JobType, WorkerStatus};
use shared::protos::intra_cluster::{
    WorkerHeartbeatResponse_HeartbeatStatus, WorkerTaskSubmissionRequest_MapType,
};
use shared::protos::user_cluster::{JobStatus_Status, MapAction_MapType};
use shared::util::random_alphanum_string;

pub fn convert_worker_status(status: &WorkerHeartbeatResponse_HeartbeatStatus) -> WorkerStatus {
    return match status {
        WorkerHeartbeatResponse_HeartbeatStatus::AWAITING_TASK => WorkerStatus::Awaiting,
        WorkerHeartbeatResponse_HeartbeatStatus::PROCESSING_TASK => WorkerStatus::Processing,
        WorkerHeartbeatResponse_HeartbeatStatus::HALTED_TASK => WorkerStatus::Halted,
        WorkerHeartbeatResponse_HeartbeatStatus::CANCELLED_TASK => WorkerStatus::Cancelled,
    };
}

pub fn split_and_validate_server(input: String) -> Option<Server> {
    let mut parts: Vec<&str> = input.split(":").collect();

    if parts.len() != 2 {
        warn!("{} does not have 2 parts!", &input);
        return None;
    }

    let port = parts.pop().unwrap();
    let hostname = parts.pop().unwrap();

    return Some(Server {
        ip: hostname.to_string(),
        port: port.parse::<i32>().unwrap(),
    });
}

pub fn unique_id<T>(items: &Arc<CHashMap<String, T>>) -> String {
    let mut user_id = random_alphanum_string(5);

    while items.contains_key(&user_id) {
        user_id = random_alphanum_string(5);
    }

    return user_id;
}

pub fn vec_remove(vec: &mut Vec<String>, to_remove: String) {
    match vec.iter().position(|x| *x == to_remove) {
        Some(index) => {
            vec.remove(index);
            ()
        }
        None => (),
    }
}

pub fn convert_job_status(status: &JobStatus) -> JobStatus_Status {
    return match status {
        JobStatus::Blocked => JobStatus_Status::BLOCKED,
        JobStatus::Running => JobStatus_Status::RUNNING,
        JobStatus::Completed => JobStatus_Status::COMPLETED,
        JobStatus::Halted => JobStatus_Status::HALTED,
        JobStatus::Cancelled => JobStatus_Status::CANCELLED,
    };
}

pub fn current_secs() -> u64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
}

pub fn convert_map_type(map_type: &MapAction_MapType) -> JobType {
    return match map_type {
        MapAction_MapType::SINGLE_IN_SINGLE_OUT => JobType::SingleInSingleOut,
        MapAction_MapType::MULTI_IN_SINGLE_OUT => JobType::MultiInSingleOut,
        MapAction_MapType::SINGLE_IN_MULTI_OUT => JobType::SingleInMultiOut,
    };
}

pub fn convert_map_task_type(job_type: &JobType) -> WorkerTaskSubmissionRequest_MapType {
    return match job_type {
        JobType::SingleInSingleOut => WorkerTaskSubmissionRequest_MapType::SINGLE_IN_SINGLE_OUT,
        JobType::MultiInSingleOut => WorkerTaskSubmissionRequest_MapType::MULTI_IN_SINGLE_OUT,
        JobType::SingleInMultiOut => WorkerTaskSubmissionRequest_MapType::SINGLE_IN_MULTI_OUT,
    };
}
