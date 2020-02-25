/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::sync::{Arc, RwLock};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use log::{error, info};
use rand::Rng;

use config::WorkerStatus;
use config::{Config, WorkerState};
use executor::docker::DockerExecutor;
use executor::shell::ShellExecutor;
use executor::{Executor, ServerMessage, ServerMessageType, TaskCommand, TaskResult};
use protobuf::RepeatedField;
use tokio::runtime::current_thread::Runtime;
use util;

pub fn start_executor(
    config: &Config,
    state: Arc<RwLock<WorkerState>>,
    master_sender: Sender<ServerMessage>,
    executor_receiver: Receiver<TaskCommand>,
) {
    const SLEEP_TIME: Duration = Duration::from_millis(300);

    let mut executor: Option<Box<Executor>> = None;
    let mut rng = rand::thread_rng();

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

    let mut loop_count: u64 = 0;

    loop {
        let mut rt = Runtime::new().unwrap();
        match executor_receiver.recv_timeout(SLEEP_TIME) {
            Ok(TaskCommand::StartTask(docker_name)) => {
                info!("Received TaskCommand::StartTask");
                if executor.is_some() {
                    error!("A task is already running!");
                } else {
                    if docker_name.len() > 0 {
                        match DockerExecutor::start_job(
                            &config,
                            &mut rt,
                            &state,
                            &master_sender,
                            &docker_name,
                        ) {
                            Some(exec) => {
                                executor = Some(Box::new(exec));
                            }
                            None => {
                                executor = None;
                            }
                        }
                    } else {
                        match ShellExecutor::start_job(
                            &config,
                            &mut rt,
                            &state,
                            &master_sender,
                            &docker_name,
                        ) {
                            Some(exec) => {
                                executor = Some(Box::new(exec));
                            }
                            None => {
                                executor = None;
                            }
                        }
                    }
                }
            }
            Ok(TaskCommand::SetNone) => {
                info!("Received TaskCommand::SetNone");
                executor = None;
            }
            Ok(TaskCommand::CancelTask) => {
                info!("Received TaskCommand::CancelTask");
                if executor.is_some() {
                    {
                        let mut exec = executor.unwrap();
                        exec.cancel_job();
                    }
                    state.write().unwrap().status = WorkerStatus::Halted;
                    executor = None;
                } else {
                    error!("Attempting to cancel a task that isn't running!");
                }
                util::restart_worker_no_send(&config, &state, &master_sender);
            }
            Err(_) => {}
        }

        if executor.is_some() {
            let mut exe = executor.unwrap();
            if exe.detect_crash(&mut rt) {
                error!("Crash detected!");
                state.write().unwrap().status = WorkerStatus::Halted;
                master_sender
                    .send(ServerMessage {
                        message_type: ServerMessageType::FinishedRequest(
                            TaskResult::JobErrored,
                            Arc::new(RepeatedField::default()),
                        ),
                        retry_count: 0,
                    })
                    .expect("Could not send finished request. Internal message broker is broken!");
                executor = None;
            } else {
                executor = Some(exe);
            }
        }

        if (loop_count % 2) == 0 {
            let unwrapped_state = state.read().unwrap();

            let current_time = util::current_secs();
            let diff = current_time - unwrapped_state.last_request;
            if diff > config.timeout.clone() as u64 {
                error!(
                    "Worker has timed out after {} seconds of no requests from master!",
                    &diff
                );
                std::process::exit(-2);
            }
        }

        loop_count += 1;
    }
}
