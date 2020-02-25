/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use executor::TaskType;
use protobuf::RepeatedField;

#[derive(Clone, Debug)]
pub struct ConfigServer {
    pub hostname: String,
    pub port: i32,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub master: ConfigServer,
    pub worker: ConfigServer,
    pub executor: ConfigServer,
    pub single_run_mode: bool,
    pub timeout: i32,
}

impl Default for Config {
    fn default() -> Config {
        return Config {
            master: ConfigServer {
                hostname: "localhost".to_string(),
                port: 1240,
            },
            worker: ConfigServer {
                hostname: "0.0.0.0".to_string(),
                port: 1242,
            },
            executor: ConfigServer {
                hostname: "0.0.0.0".to_string(),
                port: 1100,
            },
            single_run_mode: false,
            timeout: 60,
        };
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum WorkerStatus {
    Disconnected,
    Awaiting,
    Processing,
    Halted,
    Finishing,
}

#[derive(PartialEq, Clone)]
pub struct WorkerState {
    pub status: WorkerStatus,
    pub worker_id: String,
    pub last_request: u64,
    pub data_in: Option<RepeatedField<Vec<u8>>>,
    pub closure: Option<Vec<u8>>,
    pub task_type: Option<TaskType>,
    pub task_id: String,
}
