/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::sleep;
use std::time::Duration;

use atomic_counter::{AtomicCounter, ConsistentCounter};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chashmap::CHashMap;
use futures::lazy;
use log::{error, info, warn};
use protobuf::{CodedOutputStream, Message};
use tokio_threadpool::Sender;

use consensus::client::ConsensusUpdate;
use shared::protos::consensus::*;
use shared::protos::intra_cluster::ConsensusRequest_Action;
use shared::util as sutil;
use shared::BoolWrapper;
use state::State;
use util;

fn send_individual_request(
    message_id: String,
    connect_str: String,
    counter_id: &u32,
    message: Arc<Box<Message>>,
    result_counts: Arc<Mutex<HashMap<u64, usize>>>,
    data: Arc<Mutex<HashMap<u64, Vec<u8>>>>,
) -> Result<u64, ()> {
    let stream_res = TcpStream::connect(connect_str);
    if let Ok(mut stream) = stream_res {
        //Send
        {
            let size = message.compute_size();
            stream.write_u32::<BigEndian>(size.clone()); // Size
            stream.write_u32::<BigEndian>(*counter_id); // Counter ID
            let mut output_stream = CodedOutputStream::new(&mut stream);
            message.write_to(&mut output_stream).unwrap();
            output_stream.flush().unwrap();
        }

        // Receive
        let size_option = stream.read_u32::<BigEndian>();
        if size_option.is_err() {
            error!("{} || Could not read size from stream!", &message_id);
            return Err(());
        }
        let size = size_option.unwrap();
        let mut buffer = vec![0u8; size as usize];

        if let Ok(_) = stream.read_exact(&mut buffer) {
            let mut hasher = DefaultHasher::new();
            buffer.hash(&mut hasher);
            let hash = hasher.finish();

            {
                let mut lock1 = result_counts.lock().unwrap();
                let counter = lock1.entry(hash).or_insert(0);
                *counter += 1;
            }

            {
                let mut lock2 = data.lock().unwrap();
                if !lock2.contains_key(&hash) {
                    lock2.insert(hash.clone(), buffer);
                }
            }
            return Ok(hash);
        } else {
            error!(
                "{} || Could not read actual content from stream!",
                &message_id
            );
            return Err(());
        }
    } else {
        error!(
            "{} || Could not connect to master host! Error: {}",
            &message_id,
            stream_res.unwrap_err().to_string()
        );
        return Err(());
    }
}

fn drop_offending_masters(
    message_id: &String,
    max_hash: &u64,
    state: &Arc<RwLock<State>>,
    result_mapping: &Arc<CHashMap<i32, u64>>,
    client_sender: crossbeam_channel::Sender<ConsensusUpdate>,
) {
    info!("{} || Dropping offending masters!", &message_id);
    let need_new_master = Arc::new(Mutex::new(BoolWrapper::new(false)));
    let mut writable_state = state.write().unwrap();
    writable_state.masters.retain(|v| {
        if result_mapping.contains_key(&v.id)
            && result_mapping.get(&v.id).unwrap().clone() == *max_hash
        {
            true
        } else {
            if v.active {
                info!("Dropping master (leader) {}", v.id);
                need_new_master.lock().unwrap().set_value(true);
            } else {
                info!("Dropping master (not leader) {}", v.id);
            }
            client_sender
                .send(ConsensusUpdate::new_master_action_message(
                    v,
                    ConsensusRequest_Action::SHUTDOWN,
                ))
                .expect("Internal message broker is broken!");
            false
        }
    });

    if need_new_master.lock().unwrap().get_value() {
        let master = writable_state.masters.first_mut().unwrap();
        master.active = true;
        info!("Assigning {} as new leader!", &master.id);
        client_sender
            .send(ConsensusUpdate::new_master_action_message(
                master,
                ConsensusRequest_Action::SET_ACTIVE,
            ))
            .expect("Internal message broker is broken!");
    }
}

pub fn send_user_request(
    message_id: &String,
    message: Arc<Box<Message>>,
    counter_id: u32,
    state: &Arc<RwLock<State>>,
    sender: Arc<Sender>,
    client_sender: crossbeam_channel::Sender<ConsensusUpdate>,
) -> Result<Vec<u8>, ()> {
    return send_consensus_request(
        message_id,
        message,
        counter_id,
        state,
        sender,
        true,
        client_sender,
    );
}

pub fn send_worker_request(
    message_id: &String,
    message: Arc<Box<Message>>,
    counter_id: u32,
    state: &Arc<RwLock<State>>,
    sender: Arc<Sender>,
    client_sender: crossbeam_channel::Sender<ConsensusUpdate>,
) -> Result<Vec<u8>, ()> {
    return send_consensus_request(
        message_id,
        message,
        counter_id,
        state,
        sender,
        false,
        client_sender,
    );
}

fn send_consensus_request(
    message_id: &String,
    message: Arc<Box<Message>>,
    counter_id: u32,
    state: &Arc<RwLock<State>>,
    sender: Arc<Sender>,
    user: bool,
    client_sender: crossbeam_channel::Sender<ConsensusUpdate>,
) -> Result<Vec<u8>, ()> {
    let masters = state.read().unwrap().masters.clone();
    let n_requests = masters.len();
    let counter = Arc::new(ConsistentCounter::new(0));

    let result_mapping: Arc<CHashMap<i32, u64>> = Arc::new(CHashMap::new());

    let result_counts: Arc<Mutex<HashMap<u64, usize>>> = Arc::new(Mutex::new(HashMap::new()));
    let data: Arc<Mutex<HashMap<u64, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));

    for master in masters.iter() {
        let counter = counter.clone();
        let data = data.clone();
        let message = message.clone();

        let result_mapping = result_mapping.clone();
        let result_counts = result_counts.clone();

        let port = {
            if user {
                &master.user_port
            } else {
                &master.worker_port
            }
        };

        let connect_str = format!("{}:{}", &master.ip, &port);
        let ind_message_id = format!("{}-{}", &message_id, &master.ip);
        let master_id = master.id.clone();
        sender.spawn(lazy(move || {
            if let Ok(count) = send_individual_request(
                ind_message_id,
                connect_str,
                &counter_id,
                message,
                result_counts,
                data,
            ) {
                result_mapping.insert(master_id, count);
            }
            counter.inc();
            Ok(())
        }));
    }

    while counter.get() < n_requests {
        sleep(Duration::from_millis(100));
    }

    let mut data = data.lock().unwrap();
    let result_counts = result_counts.lock().unwrap();

    if result_counts.len() == 0 {
        warn!("{} || No responses received!", &message_id);
        return Err(());
    } else if result_counts.len() != 1 {
        warn!("{} || Consensus not achieved!", &message_id);

        let (max_hash, count) = result_counts
            .iter()
            .max_by(|(_i, j), (_k, l)| j.cmp(l))
            .unwrap();
        info!(
            "{} || Decided on message by {} masters",
            &message_id, *count
        );

        drop_offending_masters(
            &message_id,
            max_hash,
            &state,
            &result_mapping,
            client_sender,
        );
        return Ok(data.remove(max_hash).unwrap());
    } else {
        let max_hash = result_counts.keys().next().unwrap();
        let no_of_masters = result_counts.get(&max_hash).unwrap();
        info!(
            "{} || Consensus achieved: Decided on message by {} masters",
            &message_id, &no_of_masters
        );

        if masters.len() != *no_of_masters {
            warn!("{} || Not all masters provided a response!", &message_id);
            drop_offending_masters(
                &message_id,
                max_hash,
                &state,
                &result_mapping,
                client_sender,
            );
        }
        return Ok(data.remove(max_hash).unwrap());
    }
}

pub fn get_conflicting_id(message_id: &String, state: &Arc<RwLock<State>>) -> Result<u32, ()> {
    let leader_option = state.read().unwrap().leader.clone();
    if let Some(leader) = leader_option {
        info!(
            "{} || Getting conflicting ID from consensus leader",
            &message_id
        );

        let stream_res = TcpStream::connect(format!("{}:{}", leader.ip, leader.port));
        if stream_res.is_err() {
            error!(
                "{} || Could not connect to consensus leader! Error: {}",
                &message_id,
                stream_res.unwrap_err().to_string()
            );
            return Err(());
        }
        let mut stream = stream_res.unwrap();

        {
            let mut message = SingleConsensusRequest::new();
            message.set_conflicting_action_request(ConflictingActionRequest::new());
            let size = message.compute_size();
            stream.write_u32::<BigEndian>(size.clone());
            let mut output_stream = CodedOutputStream::new(&mut stream);
            message.write_to(&mut output_stream).unwrap();
            output_stream.flush().unwrap();
        }

        let mut message = SingleConsensusResponse::new();
        match sutil::process_input(&mut stream, &mut message, false) {
            Ok(_) => match message.response {
                Some(SingleConsensusResponse_oneof_response::conflicting_action_response(x)) => {
                    return Ok(x.id as u32);
                }
                _ => {
                    warn!("{} || Server sent an invalid response.", &message_id);
                }
            },
            Err(e) => {
                warn!(
                    "{} || Could decode message from TCP stream, Error: {}",
                    &message_id,
                    e.to_string()
                );
            }
        }

        return Err(());
    } else {
        info!(
            "{} || Instance is consensus leader, retrieving locally",
            &message_id
        );
        return Ok(state.write().unwrap().conflicting_counter.inc() as u32);
    }
}

pub fn get_unique_id(message_id: &String, state: &Arc<RwLock<State>>) -> Result<String, ()> {
    let leader_option = state.read().unwrap().leader.clone();
    if let Some(leader) = leader_option {
        info!(
            "{} || Getting conflicting ID from consensus leader",
            &message_id
        );

        let stream_res = TcpStream::connect(format!("{}:{}", leader.ip, leader.port));
        if stream_res.is_err() {
            error!(
                "{} || Could not connect to consensus leader! Error: {}",
                &message_id,
                stream_res.unwrap_err().to_string()
            );
            return Err(());
        }
        let mut stream = stream_res.unwrap();

        {
            let mut message = SingleConsensusRequest::new();
            message.set_unique_id_request(UniqueIdRequest::new());
            let size = message.compute_size();
            stream.write_u32::<BigEndian>(size.clone());
            let mut output_stream = CodedOutputStream::new(&mut stream);
            message.write_to(&mut output_stream).unwrap();
            output_stream.flush().unwrap();
        }

        let mut message = SingleConsensusResponse::new();
        match sutil::process_input(&mut stream, &mut message, false) {
            Ok(_) => match message.response {
                Some(SingleConsensusResponse_oneof_response::unique_id_response(x)) => {
                    return Ok(x.id);
                }
                _ => {
                    warn!("{} || Server sent an invalid response.", &message_id);
                }
            },
            Err(e) => {
                warn!(
                    "{} || Could decode message from TCP stream, Error: {}",
                    &message_id,
                    e.to_string()
                );
            }
        }

        return Err(());
    } else {
        info!(
            "{} || Instance is consensus leader, retrieving locally",
            &message_id
        );
        let mut writable_state = state.write().unwrap();
        let id = util::unique_id(&writable_state.unique_id_source);
        writable_state.unique_id_source.insert(id.clone());
        return Ok(id);
    }
}
