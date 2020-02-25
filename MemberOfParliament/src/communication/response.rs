/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::sync::{RwLock, Arc};

use log::{info, warn};
use crossbeam_channel::Sender;

use config::{Config, WorkerState, WorkerStatus};
use protos::intra_cluster::{WorkerFinishedResponse, WorkerConnectionResponse};
use executor::{TaskCommand, ServerMessage};
use util;

pub trait ResponseHandler {
    fn handle_message(&mut self,
                      config: &Config,
                      message_id: &String,
                      state: &Arc<RwLock<WorkerState>>,
                      master_sender: &Sender<ServerMessage>,
                      executor_sender: &Sender<TaskCommand>) -> bool;
}

impl ResponseHandler for WorkerConnectionResponse {
    fn handle_message(&mut self,
                      _config: &Config,
                      message_id: &String,
                      state: &Arc<RwLock<WorkerState>>,
                      _master_sender: &Sender<ServerMessage>,
                      _executor_sender: &Sender<TaskCommand>) -> bool {
        info!("{} || Processing message as a WorkerConnectionResponse", &message_id);

        if self.connection_accepted {
            info!("{} || Worker connection has been accepted!", &message_id);

            let mut worker_state = state.write().unwrap();
            worker_state.worker_id = self.take_worker_id();
            worker_state.status = WorkerStatus::Awaiting;
        } else {
            warn!("{}|| Worker connection has not been accepting. Closing...", &message_id);
            panic!();
        }
        return true;
    }
}

impl ResponseHandler for WorkerFinishedResponse {
    fn handle_message(&mut self,
                      config: &Config,
                      message_id: &String,
                      state: &Arc<RwLock<WorkerState>>,
                      master_sender: &Sender<ServerMessage>,
                      executor_sender: &Sender<TaskCommand>) -> bool {
        info!("{} || Processing message as a WorkerFinishedResponse", &message_id);
        if self.response_processed {
            util::restart_worker(&config, &state, &master_sender, &executor_sender);
            return true;
        } else {
            info!("{} || Response has not been processed. Retrying...", &message_id);
            return false;
        }
    }
}