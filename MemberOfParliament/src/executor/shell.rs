/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::{env, thread};
use std::sync::{Arc, RwLock};
use std::io::{BufReader, BufRead};
use std::process::{Stdio, Child, Command};

use crossbeam_channel::Sender;
use protobuf::RepeatedField;
use log::{error, info};

use tokio::runtime::current_thread::Runtime;
use config::{Config, WorkerStatus, WorkerState};
use executor::{Executor, TaskResult, ServerMessage, ServerMessageType};


pub struct ShellExecutor {
    pub process : Child
}

impl Executor for ShellExecutor {

    fn start_job(config: &Config,
                 _rt: &mut Runtime,
                 state: &Arc<RwLock<WorkerState>>,
                 master_sender: &Sender<ServerMessage>,
                 _docker_name: &String) -> Option<ShellExecutor> {
        info!("Running using ShellExecutor");
        let home_dir = env::home_dir().unwrap();
        let command_option = Command::new(format!("{}/executables/parallel.exe", home_dir.display()))
            .env("PARLIAMENT_MODE".to_string(),"Worker".to_string())
            .env("PARLIAMENT_HOST".to_string(), "localhost".to_string())
            .env("PARLIAMENT_PORT".to_string(), config.executor.port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        if command_option.is_err() {
            let err = command_option.unwrap_err();
            error!("Received error when attempting to execute process: {}", err.to_string());
            state.write().unwrap().status = WorkerStatus::Halted;
            master_sender.send(ServerMessage {
                message_type: ServerMessageType::FinishedRequest(TaskResult::JobErrored, Arc::new(RepeatedField::default())),
                retry_count: 0
            }).expect("Could not send finished request. Internal message broker is broken!");
            return None;
        }

        state.write().unwrap().status = WorkerStatus::Processing;

        let mut command = command_option.unwrap();

        let stdout = command.stdout.take().unwrap();
        let stderr = command.stderr.take().unwrap();

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            reader
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| println!("{}", line));

            let reader = BufReader::new(stderr);
            reader
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| println!("{}", line));
        });

        return Some(ShellExecutor {
            process: command
        });
    }

    fn cancel_job(&mut self) {
        self.process.kill();
    }

    fn detect_crash(&mut self, _rt: &mut Runtime) -> bool {
        match self.process.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    error!("Process ended in with an unsuccessful error code! Error code: {}", status.code().unwrap());
                    return true;
                }
            },
            _ => return false
        }
        return false;
    }
}
