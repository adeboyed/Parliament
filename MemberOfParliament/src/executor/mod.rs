/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use protobuf::RepeatedField;
use std::sync::Arc;
use config::WorkerState;
use crossbeam_channel::Sender;
use std::sync::RwLock;
use config::Config;
use tokio::runtime::current_thread::Runtime;

pub mod executor;
pub mod docker;
pub mod shell;
pub mod communication;

#[derive(PartialEq,Clone,Debug)]
pub enum TaskType {
    SingleInMultiOut,
    SingleInSingleOut,
    MultiInSingleOut
}

#[derive(PartialEq, Clone)]
pub enum TaskCommand {
    StartTask(String),
    CancelTask,
    SetNone,
}

#[derive(PartialEq, Clone)]
pub enum TaskResult {
    JobFinished,
    JobErrored
}

#[derive(PartialEq, Clone)]
pub enum ServerMessageType {
    ConnectionRequest(String, i32),
    FinishedRequest(TaskResult, Arc<RepeatedField<Vec<u8>>>)
}

pub struct ServerMessage {
    pub message_type: ServerMessageType,
    pub retry_count: i32
}


pub trait Executor {
    fn start_job(config: &Config,
                 runtime: &mut Runtime,
                 state: &Arc<RwLock<WorkerState>>,
                 master_sender: &Sender<ServerMessage>,
                 docker_name: &String) -> Option<Self> where Self : Sized;

    fn cancel_job(&mut self);

    fn detect_crash(&mut self, runtime: &mut Runtime) -> bool;
}