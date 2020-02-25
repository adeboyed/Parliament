/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;

use crossbeam_channel::Sender;
use protobuf::RepeatedField;
use log::info;
use shiplift::{ContainerOptions, Docker, Container};
use shiplift::rep::ContainerDetails;
use tokio::prelude::Future;
use tokio::runtime::current_thread::Runtime;

use config::{Config, WorkerState, WorkerStatus};
use executor::{Executor, TaskResult, ServerMessage, ServerMessageType};
use core::mem;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DockerExecutor {
    pub id : String,
    pub docker : Docker,
}

fn current_millis() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    return since_the_epoch.as_secs() * 1000 +
        since_the_epoch.subsec_nanos() as u64 / 1_000_000;
}

impl Executor for DockerExecutor {

    fn start_job(config: &Config,
                 rt: &mut Runtime,
                 state: &Arc<RwLock<WorkerState>>,
                 master_sender: &Sender<ServerMessage>,
                 docker_name: &String) -> Option<DockerExecutor> {
        info!("Running using DockerExecutor");
        let worker_id = state.read().unwrap().worker_id.clone();
        let docker = Docker::new();

        // Check image exists

        let mut labels = HashMap::new();
        labels.insert("parliament", "worker");
        labels.insert("worker-id", &worker_id);

        let port_str = format!("PARLIAMENT_PORT={}", config.executor.port);

        let mut envs = Vec::new();
        envs.push("PARLIAMENT_MODE=Worker");
        envs.push("PARLIAMENT_HOST=host.docker.internal");
        envs.push(&port_str);

        let options = ContainerOptions::builder(&docker_name)
            .env(envs)
            .name(&worker_id)
            .labels(&labels)
            .auto_remove(false)
            .build();

        let id = Arc::new(Mutex::new("".to_string()));

        let id_clone = id.clone();

        let before = current_millis();
        let fut = docker
            .containers()
            .create(&options)
            .map(|info| {
                let id = info.id;
                info!("Created container ID: {}", &id);
                let mut id_state = id_clone.lock().expect("Could not lock mutex");
                *id_state = id.clone();
                let container = Container::new(&docker, id);

                // We have to say that we are Processing before we even start the container in case
                // Container finishes execution before tokio cleans up
                state.write().unwrap().status = WorkerStatus::Processing;

                return container;
            })
            .and_then(|container: Container| container.start());

        rt.block_on(fut);

        let after = current_millis();
        let diff = after - before;
        info!("Time taken to create container: {}", diff);


        let mut id_state = id.lock().expect("Could not lock mutex");
        let id = mem::replace(&mut *id_state, String::from(""));
//        state.write().unwrap().status = WorkerStatus::Processing;
        if id.len() > 0 {
            return Some(DockerExecutor {
                id: id.clone(),
                docker: Docker::new()
            });
        }

        state.write().unwrap().status = WorkerStatus::Halted;
        master_sender.send(ServerMessage {
            message_type: ServerMessageType::FinishedRequest(TaskResult::JobErrored, Arc::new(RepeatedField::default())),
            retry_count: 0
        }).expect("Could not send finished request. Internal message broker is broken!");
        return None;
    }

    fn cancel_job(&mut self) {
        let container = Container::new(&self.docker, &self.id);
        container.kill(None);
    }

    fn detect_crash(&mut self, rt: &mut Runtime) -> bool {
        let container = Container::new(&self.docker, &self.id);
        match rt.block_on(container.inspect()) {
            Ok(details) => {
                let state = details.state;
                return !state.running && state.exit_code != 0;
            },
            Err(_) => ()
        }
        return false;
    }
}
