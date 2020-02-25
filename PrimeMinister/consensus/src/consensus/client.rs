/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::sync::{Arc, RwLock};
use std::net::TcpStream;
use std::thread::{Builder, JoinHandle};

use log::{info, warn, error};
use byteorder::{WriteBytesExt, BigEndian};
use crossbeam_channel::{Sender, Receiver};
use protobuf::{CodedOutputStream, Message, RepeatedField};

use shared::protos::consensus::*;
use shared::protos::intra_cluster::*;
use shared::util;
use state::State;
use consensus::response::ConsensusResponseHandler;
use state::ConsensusMachine;
use state::MasterMachine;

#[derive(Clone)]
pub enum UpdateType {
    Consensus(SingleConsensusRequest),
    Master(SingleWorkerMessage),
}

#[derive(Clone)]
pub struct ConsensusUpdate {
    pub update_type: UpdateType,
    pub id: i32,
    pub ip_addr: String,
    pub ip_port: i32,
    pub retry_count: i32,
}

impl ConsensusUpdate {
    pub fn new_leader_connection(consensus: ConsensusMachine, port: i32) -> ConsensusUpdate {
        let mut request = SingleConsensusRequest::new();
        let mut leader_request = LeaderConnectionRequest::new();
        leader_request.set_port(port);
        request.set_leader_connection_request(leader_request);
        return ConsensusUpdate::new_consensus(consensus, request);
    }

    pub fn new_heartbeat_message(consensus: ConsensusMachine, consensuses: Vec<Consensus>, masters: Vec<Master>) -> ConsensusUpdate {
        let mut request = SingleConsensusRequest::new();
        let mut heartbeat_request = HeartbeatRequest::new();
        heartbeat_request.set_masters(RepeatedField::from_vec(masters));
        heartbeat_request.set_consensuses(RepeatedField::from_vec(consensuses));
        request.set_heartbeat_request(heartbeat_request);
        return ConsensusUpdate::new_consensus(consensus, request);
    }

    pub fn new_master_action_message(master: &MasterMachine, action: ConsensusRequest_Action) -> ConsensusUpdate {
        let mut single_message = SingleWorkerMessage::new();
        let mut request = ConsensusRequest::new();
        request.set_action(action);
        single_message.set_consensus_request(request);
        return ConsensusUpdate::new_master(master, single_message);
    }

    pub fn new_master(master: &MasterMachine, message: SingleWorkerMessage) -> ConsensusUpdate {
        return ConsensusUpdate {
            update_type: UpdateType::Master(message),
            id: master.id.clone(),
            ip_addr: master.ip.clone(),
            ip_port: master.worker_port.clone(),
            retry_count: 1
        }
    }

    pub fn new_consensus(consensus: ConsensusMachine, request: SingleConsensusRequest) -> ConsensusUpdate {
        return ConsensusUpdate {
            update_type: UpdateType::Consensus(request),
            id: consensus.id,
            ip_addr: consensus.ip,
            ip_port: consensus.port,
            retry_count: 1,
        };
    }
}

fn retry_message(mut message: ConsensusUpdate,
                 message_id: &String,
                 state: &Arc<RwLock<State>>,
                 sender: &Sender<ConsensusUpdate>) {
    error!("{} || Failed to receive a correct response from the server. Retrying...", &message_id);
    if message.retry_count < 3 {
        message.retry_count = message.retry_count + 1;
        sender.send(message).expect("Internal message broker has crashed!")
    } else {
        let mut writable_state = state.write().unwrap();
        error!("{} || Attempting to send a message 3 times! Removing Consensus with ID:  {}", &message_id, &message.id);
        writable_state.consensuses.retain(|x| x.id != message.id);

        if let Some(leader) = &writable_state.leader {
            if &message.ip_addr == &leader.ip && &message.ip_port == &leader.port {
                error!("Leader has died, ending...");
                std::process::exit(2);
            }
        }
    }
}

fn handle_consensus_message(update: ConsensusUpdate,
                            message: &SingleConsensusRequest,
                            sender: &Sender<ConsensusUpdate>,
                            state: &Arc<RwLock<State>>,
                            mut stream: TcpStream,
                            message_id: String) {
    {
        let size = message.compute_size(); // TODO Should error check!
        stream.write_u32::<BigEndian>(size.clone());
        let mut output_stream = CodedOutputStream::new(&mut stream);
        message.write_to(&mut output_stream).unwrap();
        output_stream.flush().unwrap();
    }

    let mut message = SingleConsensusResponse::new();
    let retry: bool = match util::process_input(&mut stream, &mut message, false) {
        Ok(_) => {
            match message.response {
                Some(SingleConsensusResponse_oneof_response::conflicting_action_response(mut x)) => x.handle_message(&message_id, &state),
                Some(SingleConsensusResponse_oneof_response::leader_connection_response(mut x)) => x.handle_message(&message_id, &state),
                Some(SingleConsensusResponse_oneof_response::heartbeat_response(mut x)) => x.handle_message(&message_id, &state),
                Some(SingleConsensusResponse_oneof_response::not_leader_response(mut x)) => x.handle_message(&message_id, &state),
                Some(SingleConsensusResponse_oneof_response::unique_id_response(mut x)) => x.handle_message(&message_id, &state),
                None => {
                    warn!("{} || Server sent an empty (valid) response.", &message_id);
                    false
                }
            }
        }
        Err(e) => {
            warn!("{} || Could decode message from TCP stream, Error: {}", &message_id, e.to_string());
            false
        }
    };
    if !retry {
        retry_message(update, &message_id, &state, &sender);
    }
}

fn handle_master_message(message: &SingleWorkerMessage,
                         mut stream: TcpStream) {
    let size = message.compute_size(); // TODO Should error check!
    stream.write_u32::<BigEndian>(size.clone());
    stream.write_u32::<BigEndian>(0);
    let mut output_stream = CodedOutputStream::new(&mut stream);
    message.write_to(&mut output_stream).unwrap();
    output_stream.flush().unwrap();
}


pub fn start(state: Arc<RwLock<State>>,
             sender: Sender<ConsensusUpdate>,
             receiver: Receiver<ConsensusUpdate>) -> Vec<JoinHandle<()>> {
    let mut client_threads = vec![];
    for i in 0..2 {
        let sender = sender.clone();
        let receiver = receiver.clone();
        let state = state.clone();
        client_threads.push(Builder::new().name(format!("{}-{}", "client", i)).spawn(move || {
            loop {
                let update = receiver.recv().expect("Internal message broker has crashed!");
                let message_id = util::random_alphanum_string(10);

                let stream_res = TcpStream::connect(format!("{}:{}", update.ip_addr, update.ip_port));
                if stream_res.is_err() {
                    error!("{} || Could not connect to host! Error: {}", &message_id, stream_res.unwrap_err().to_string());
                    retry_message(update, &message_id, &state, &sender);
                    continue;
                }
                let stream = stream_res.unwrap();

                match &update.update_type {
                    UpdateType::Consensus(message) => {
                        info!("{} || UpdateType::Consensus", &message_id);
                        handle_consensus_message(update.clone(), message, &sender, &state, stream, message_id);
                    }
                    UpdateType::Master(message) => {
                        info!("{} || UpdateType::Master", &message_id);
                        handle_master_message(message, stream);
                    }
                }
            }
        }).expect("Could not create client thread!"));
    }
    return client_threads;
}