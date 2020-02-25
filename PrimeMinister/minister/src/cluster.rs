/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
    
    cluster.rs - Performs all of the operations between users and workers
    Also manages everything
*/

use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use std::{thread, time};

use chashmap::CHashMap;
use crossbeam::queue::MsQueue;
use crossbeam_channel::Sender;
use log::{error, info, warn};

use config::State;
use model::JobStatus;
use model::{JobType, TaskStatus, WJob, WTask, Worker, WorkerStatus, WorkerUpdate};
use std::sync::Mutex;
use users::User;
use util;

fn kick_inactive_users(users: &Arc<CHashMap<String, User>>) {
    let oldest_acceptable_time = util::current_secs() - 100;
    let jobs_to_del = Arc::new(Mutex::new(Vec::new()));

    users.retain(|k, v| {
        if v.last_request > oldest_acceptable_time {
            return true;
        } else {
            warn!("Kicking user {}", k);
            v.jobs
                .iter()
                .for_each(|x| jobs_to_del.lock().unwrap().push(x.clone()));
            return false;
        }
    });

    //    let tasks = jobs_to_del.lock().unwrap()
    //        .iter()
    //        .map(|x| jobs.get(&x).unwrap())
    //        .map(|x| x.tasks)
    //        .flatten()
    //        .collect::Vec<String>>();

    // TODO Will need to clear and pre-empt any running tasks and clear all data
}

fn create_tasks_from_queued_jobs(
    jobs: &Arc<CHashMap<String, WJob>>,
    data: &Arc<CHashMap<String, Vec<Vec<u8>>>>,
    jobs_queue: &Arc<MsQueue<String>>,
    tasks: &Arc<CHashMap<String, WTask>>,
    tasks_queue: &Arc<MsQueue<String>>,
) {
    while !jobs_queue.is_empty() {
        let job_id = jobs_queue.pop();
        info!("Processing job {} from job queue!", job_id);
        let job_option = jobs.get_mut(&job_id);
        if let Some(mut job) = job_option {
            if job.status != JobStatus::Blocked {
                warn!("Job is not in Blocked status, skipping...");
                continue;
            }

            match job.job_type {
                JobType::SingleInMultiOut => {
                    let task_id = format!("{}-{}", job_id, 0);
                    let task = WTask {
                        id: task_id.clone(),
                        user_id: job.user_id.clone(),
                        job_id: job_id.clone(),
                        data_in_id: job.input_job_id.clone(),
                        data_in_loc: 0,
                        data_out_id: job_id.clone(),
                        docker_name: job.docker_name.clone(),
                        data_out_loc: -1,
                        closure: job.closure.clone(),
                        status: TaskStatus::Awaiting,
                        job_type: JobType::SingleInMultiOut,
                    };

                    tasks.insert(task_id.clone(), task);
                    tasks_queue.push(task_id.clone());
                    job.tasks.insert(task_id.clone());
                    job.total_tasks = 1;
                    job.status = JobStatus::Running;
                    info!("Created task {} for job {}", task_id, job_id);
                }
                JobType::SingleInSingleOut => {
                    let input_data = match data.get(&job.input_job_id) {
                        Some(data) => data,
                        None => panic!(),
                    };
                    let no_of_input = input_data.len();
                    if no_of_input == 0 {
                        warn!("SingleInSingleOut job has 0 inputs, no tasks will be created!");
                    }
                    for i in 0..no_of_input {
                        let task_id = format!("{}-{}", job_id, i);
                        let task = WTask {
                            id: task_id.clone(),
                            user_id: job.user_id.clone(),
                            job_id: job_id.clone(),
                            data_in_id: job.input_job_id.clone(),
                            data_in_loc: i as i32,
                            data_out_id: job_id.clone(),
                            data_out_loc: i as i32,
                            closure: job.closure.clone(),
                            docker_name: job.docker_name.clone(),
                            status: TaskStatus::Awaiting,
                            job_type: JobType::SingleInSingleOut,
                        };

                        tasks.insert(task_id.clone(), task);
                        tasks_queue.push(task_id.clone());
                        job.tasks.insert(task_id.clone());
                        job.status = JobStatus::Running;
                        info!("Created task {} for job {}", task_id, job_id);
                    }
                    job.total_tasks = input_data.len().clone() as i32;
                }
                JobType::MultiInSingleOut => {
                    let task_id = format!("{}-{}", job_id, 0);
                    let task = WTask {
                        id: task_id.clone(),
                        user_id: job.user_id.clone(),
                        job_id: job_id.clone(),
                        data_in_id: job.input_job_id.clone(),
                        data_in_loc: -1,
                        data_out_id: job_id.clone(),
                        data_out_loc: 0,
                        closure: job.closure.clone(),
                        docker_name: job.docker_name.clone(),
                        status: TaskStatus::Awaiting,
                        job_type: JobType::MultiInSingleOut,
                    };

                    tasks.insert(task_id.clone(), task);
                    tasks_queue.push(task_id.clone());
                    job.tasks.insert(task_id.clone());
                    job.total_tasks = 1;
                    job.status = JobStatus::Running;
                    info!("Created task {} for job {}", job_id, task_id);
                }
            }
        } else {
            warn!("Job {} was in queue, however removed from map", job_id);
        }
    }
}

fn available_worker(
    names: &Vec<String>,
    workers: &Arc<CHashMap<String, Worker>>,
) -> Option<String> {
    for name in names {
        match workers.get_mut(name) {
            Some(mut worker) => {
                if worker.status == WorkerStatus::Awaiting && !worker.assigned {
                    worker.assigned = true;
                    return Some(worker.id.clone());
                }
            }
            None => (),
        }
    }
    None
}

fn assign_tasks_to_workers(
    worker_names: &Arc<RwLock<Vec<String>>>,
    workers: &Arc<CHashMap<String, Worker>>,
    tasks: &Arc<CHashMap<String, WTask>>,
    tasks_queue: &Arc<MsQueue<String>>,
    running_tasks: &Arc<RwLock<HashSet<String>>>,
    update_sender: &Sender<WorkerUpdate>,
) {
    loop {
        if !tasks_queue.is_empty() {
            if let Some(worker_id) = available_worker(&worker_names.read().unwrap(), workers) {
                let task_id = tasks_queue.pop();
                match tasks.get(&task_id) {
                    Some(task) => {
                        if task.status == TaskStatus::Awaiting {
                            let mut worker = workers.get_mut(&worker_id).unwrap();
                            match update_sender
                                .send(WorkerUpdate::submission(&worker, task_id.clone()))
                            {
                                Ok(_) => {
                                    worker.assigned = true;
                                    worker.running_task = Some(task_id.clone());
                                    running_tasks.write().unwrap().insert(task_id.clone());
                                }
                                Err(e) => error!(
                                    "Could not add worker_update to channel! Error: {}",
                                    e.to_string()
                                ),
                            }
                        }
                    }
                    None => (),
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

fn send_heartbeat_requests(
    worker_names: &Arc<RwLock<Vec<String>>>,
    workers: &Arc<CHashMap<String, Worker>>,
    update_sender: &Sender<WorkerUpdate>,
) {
    worker_names
        .read()
        .unwrap()
        .iter()
        .map(|x| workers.get(x).unwrap())
        .filter(|x| x.status != WorkerStatus::Cancelled)
        .filter(|x| x.status != WorkerStatus::Finishing)
        .map(|x| WorkerUpdate::heartbeat(&x))
        .map(|x| update_sender.send(x))
        .for_each(|x| match x {
            Ok(_) => (),
            Err(e) => error!(
                "Could not add worker_update to channel! Error: {}",
                e.to_string()
            ),
        });
}

fn detect_worker_crashes(
    worker_names: &Arc<RwLock<Vec<String>>>,
    running_tasks: &Arc<RwLock<HashSet<String>>>,
    workers: &Arc<CHashMap<String, Worker>>,
    tasks_queue: &Arc<MsQueue<String>>,
    tasks: &Arc<CHashMap<String, WTask>>,
) {
    worker_names.write().unwrap().retain(|worker_id| {
        let worker_option = workers.get(worker_id);
        if worker_option.is_none() {
            error!("Inconsistency found between worker_names and workers");
            return false;
        }
        let worker = worker_option.unwrap().clone();
        if worker.missed_heartbeats > 6 {
            info!("Removing worker {}, reached heartbeat limit!", worker_id);

            if let Some(task_id) = &worker.running_task {
                info!("Had to reschedule task {} running on worker!", &task_id);
                let mut task = tasks.get_mut(task_id).unwrap();
                task.status = TaskStatus::Awaiting;
                running_tasks.write().unwrap().remove(worker_id);
                tasks_queue.push(task_id.clone()); //TODO FIX: This is annoying, task placed at the back of the queue
            }
            workers.remove(&worker_id);
            return false;
        } else {
            return true;
        }
    });
}

fn handle_finished_tasks(
    running_tasks: &Arc<RwLock<HashSet<String>>>,
    data: &Arc<CHashMap<String, Vec<Vec<u8>>>>,
    tasks: &Arc<CHashMap<String, WTask>>,
    jobs_queue: &Arc<MsQueue<String>>,
    jobs: &Arc<CHashMap<String, WJob>>,
) {
    running_tasks.write().unwrap().retain(|x| {
        let task_option = tasks.get(x);
        if task_option.is_none() {
            error!("Inconsistency found between worker_names and workers");
            return false;
        }
        let task = task_option.unwrap();
        if task.status == TaskStatus::Completed {
            let mut job = jobs.get_mut(&task.job_id).unwrap();
            job.completed_tasks += 1;
            if job.completed_tasks == job.total_tasks {
                job.status = JobStatus::Completed;
                if let Some(output_job) = &job.output_job_id {
                    info!(
                        "All tasks for {} have completed, placing next job on queue",
                        &task.job_id
                    );
                    jobs_queue.push(output_job.clone());

                    info!("Going to clean up data now!");
                    data.remove(&job.input_job_id);
                } else {
                    info!("All tasks for {} have completed", &task.job_id);
                }
            } else {
                info!(
                    "Completed tasks {}/{} for job {}",
                    &job.completed_tasks, &job.total_tasks, &task.job_id
                );
            }
            return false;
        } else {
            return true;
        }
    });
}

fn handle_errored_tasks(
    running_tasks: &Arc<RwLock<HashSet<String>>>,
    workers: &Arc<CHashMap<String, Worker>>,
    tasks: &Arc<CHashMap<String, WTask>>,
    jobs: &Arc<CHashMap<String, WJob>>,
    data: &Arc<CHashMap<String, Vec<Vec<u8>>>>,
    update_sender: &Sender<WorkerUpdate>,
    consensus_mode: bool,
    consensus_state: &Arc<State>,
) {
    running_tasks.write().unwrap()
        .retain(|x| {
            let task_option = tasks.get(&x);
            if task_option.is_none() {
                error!("Inconsistency found between worker_names and workers");
                return false;
            }

            let task = task_option.unwrap().clone();
            if task.status == TaskStatus::Cancelled {
                return false;
            } else if task.status == TaskStatus::Halted {
                let mut job = jobs.get_mut(&task.job_id).unwrap();
                if job.status != JobStatus::Halted {
                    job.status = JobStatus::Halted;

                    // Cancel tasks
                    job.tasks.clone()
                        .into_iter()
                        .for_each(|x| {
                            let mut task = tasks.get_mut(&x).unwrap();
                            task.status = match task.status {
                                TaskStatus::Awaiting => TaskStatus::Cancelled,
                                TaskStatus::Completed => TaskStatus::Completed,
                                TaskStatus::Halted => TaskStatus::Cancelled,
                                TaskStatus::Cancelled => TaskStatus::Cancelled, // Shouldn't ever happen
                                TaskStatus::Running(ref worker_id) => {
                                    if let Some(worker) = workers.get(&worker_id) {
                                        if consensus_mode {
                                            if consensus_state.active.read().unwrap().get_value() {
                                                update_sender.send(WorkerUpdate::cancellation(&worker));
                                            } else {
                                                info!("CONSENSUS: Did not send cancellation message as master not active")
                                            }
                                        } else {
                                            update_sender.send(WorkerUpdate::cancellation(&worker));
                                        }
                                    }
                                    TaskStatus::Cancelled
                                }
                            };
                        });

                    
                    let mut prev_job_id = job.id.clone();

                    // Cancel all subsequent jobs
                    let mut next_job = job;
                    while next_job.output_job_id != None {
                        next_job = jobs.get_mut(&next_job.output_job_id.clone().unwrap()).unwrap();
                        next_job.status = JobStatus::Cancelled;
                    }

                    // Remove all data
                    while prev_job_id.len() > 0 {
                        data.remove(&prev_job_id);
                        prev_job_id = jobs.get(&prev_job_id).unwrap().input_job_id.clone();
                    }

                }
                return false;
            } else {
                return true;
            }
        });
}

fn cluster_stats(
    users: &Arc<CHashMap<String, User>>,
    jobs: &Arc<CHashMap<String, WJob>>,
    workers: &Arc<CHashMap<String, Worker>>,
    worker_names: &Arc<RwLock<Vec<String>>>,
    running_tasks: &Arc<RwLock<HashSet<String>>>,
    update_sender: &Sender<WorkerUpdate>,
) {
    info!("-------------------------");
    info!("----- CLUSTER STATS -----");
    info!("NO OF USERS: {}", users.len());
    info!("NO OF WORKER NAMES: {}", worker_names.read().unwrap().len());
    info!("NO OF WORKERS: {}", workers.len());
    info!("NO OF JOBS: {}", jobs.len());

    info!("RUNNING TASKS:");
    running_tasks
    .read()
    .unwrap()
    .iter()
    .for_each(|v| info!("{}", v));
    // info!("WORKERS: ");
    // worker_names
    //     .read()
    //     .unwrap()
    //     .iter()
    //     .for_each(|v| info!("{:?}", workers.get(v).unwrap().clone()));
    info!("-------------------------");
}

pub fn run(
    users: Arc<CHashMap<String, User>>,
    data: Arc<CHashMap<String, Vec<Vec<u8>>>>,
    jobs: Arc<CHashMap<String, WJob>>,
    jobs_queue: Arc<MsQueue<String>>,
    worker_names: Arc<RwLock<Vec<String>>>,
    workers: Arc<CHashMap<String, Worker>>,
    tasks: Arc<CHashMap<String, WTask>>,
    tasks_queue: Arc<MsQueue<String>>,
    running_tasks: Arc<RwLock<HashSet<String>>>,
    update_sender: Sender<WorkerUpdate>,
    consensus_mode: bool,
    consensus_state: Arc<State>,
) {
    const SLEEP_TIME: time::Duration = time::Duration::from_millis(50);

    let mut loop_count: u64 = 0;

    loop {
        if (loop_count % 5) == 0 {
            kick_inactive_users(&users);
            detect_worker_crashes(
                &worker_names,
                &running_tasks,
                &workers,
                &tasks_queue,
                &tasks,
            );
        }

        handle_finished_tasks(&running_tasks, &data, &tasks, &jobs_queue, &jobs);
        handle_errored_tasks(
            &running_tasks,
            &workers,
            &tasks,
            &jobs,
            &data,
            &update_sender,
            consensus_mode,
            &consensus_state,
        );
        create_tasks_from_queued_jobs(&jobs, &data, &jobs_queue, &tasks, &tasks_queue);

        if !consensus_mode || (consensus_mode && consensus_state.active.read().unwrap().get_value())
        {
            assign_tasks_to_workers(
                &worker_names,
                &workers,
                &tasks,
                &tasks_queue,
                &running_tasks,
                &update_sender,
            );
        }

        if (loop_count % 15) == 0 {
            send_heartbeat_requests(&worker_names, &workers, &update_sender);
        }

               if (loop_count % 10) == 0 {
                   cluster_stats(&users, &jobs, &workers, &worker_names, &running_tasks, &update_sender);
               }

        thread::sleep(SLEEP_TIME);
        loop_count = loop_count + 1;
    }
}
