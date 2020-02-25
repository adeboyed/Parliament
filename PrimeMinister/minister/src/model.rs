/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::collections::HashSet;
use std::sync::Arc;
use util;

#[derive(PartialEq, Clone, Debug)]
pub enum WorkerStatus {
    Awaiting,
    Processing,
    Halted,
    Cancelled,
    Finishing,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Worker {
    pub id: String,
    pub ip_addr: String,
    pub ip_port: i32,
    pub last_heartbeat: u64,
    pub running_task: Option<String>,
    pub status: WorkerStatus,
    pub assigned: bool,
    pub missed_heartbeats: i32,
}

impl Worker {
    pub fn new(id: String, ip_addr: String, ip_port: i32) -> Worker {
        return Worker {
            id,
            ip_addr,
            ip_port,
            last_heartbeat: util::current_secs(),
            running_task: None,
            status: WorkerStatus::Awaiting,
            assigned: false,
            missed_heartbeats: 0,
        };
    }
}

#[derive(PartialEq, Clone)]
pub enum WorkerUpdateType {
    Heartbeat,
    Cancellation,
    Submission(String),
}

#[derive(PartialEq, Clone)]
pub struct WorkerUpdate {
    pub message: WorkerUpdateType,
    pub worker_id: String,
    pub ip_addr: String,
    pub ip_port: i32,
    pub entrance_time: u64,
    pub retry_count: u32,
}

impl WorkerUpdate {
    pub fn submission(worker: &Worker, task_id: String) -> WorkerUpdate {
        return WorkerUpdate {
            message: WorkerUpdateType::Submission(task_id),
            worker_id: worker.id.clone(),
            ip_addr: worker.ip_addr.clone(),
            ip_port: worker.ip_port.clone(),
            entrance_time: util::current_secs(),
            retry_count: 2,
        };
    }

    pub fn heartbeat(worker: &Worker) -> WorkerUpdate {
        return WorkerUpdate {
            message: WorkerUpdateType::Heartbeat,
            worker_id: worker.id.clone(),
            ip_addr: worker.ip_addr.clone(),
            ip_port: worker.ip_port.clone(),
            entrance_time: util::current_secs(),
            retry_count: 2,
        };
    }

    pub fn cancellation(worker: &Worker) -> WorkerUpdate {
        return WorkerUpdate {
            message: WorkerUpdateType::Cancellation,
            worker_id: worker.id.clone(),
            ip_addr: worker.ip_addr.clone(),
            ip_port: worker.ip_port.clone(),
            entrance_time: util::current_secs(),
            retry_count: 0,
        };
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum JobType {
    SingleInMultiOut,
    SingleInSingleOut,
    MultiInSingleOut,
}

#[derive(PartialEq, Clone, Debug)]
pub enum JobStatus {
    Blocked,
    Running,
    Completed,
    Halted,
    Cancelled,
}

#[derive(PartialEq, Clone, Debug)]
pub struct WJob {
    pub id: String,
    pub user_id: String,
    pub input_job_id: String,
    pub output_job_id: Option<String>,
    pub job_type: JobType,
    pub status: JobStatus,
    pub docker_name: String,
    pub closure: Arc<Vec<u8>>,
    pub total_tasks: i32,
    pub completed_tasks: i32,
    pub tasks: HashSet<String>,
}

impl WJob {
    pub fn new(
        id: String,
        user_id: String,
        input_job_id: String,
        docker_name: String,
        job_type: JobType,
        closure: Vec<u8>,
    ) -> WJob {
        return WJob {
            id,
            user_id,
            input_job_id,
            output_job_id: None,
            job_type,
            status: JobStatus::Blocked,
            docker_name,
            closure: Arc::new(closure),
            total_tasks: 0,
            completed_tasks: 0,
            tasks: HashSet::new(),
        };
    }

    pub fn set_output_id(&mut self, output_job_id: String) {
        self.output_job_id = Some(output_job_id);
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum TaskStatus {
    Awaiting,
    Running(String),
    Completed,
    Halted,
    Cancelled,
}

#[derive(PartialEq, Clone, Debug)]
pub struct WTask {
    pub id: String,
    pub job_id: String,
    pub user_id: String,
    pub data_in_id: String,
    pub data_in_loc: i32,
    pub data_out_id: String,
    pub data_out_loc: i32,
    pub docker_name: String,
    pub status: TaskStatus,
    pub closure: Arc<Vec<u8>>,
    pub job_type: JobType,
}
