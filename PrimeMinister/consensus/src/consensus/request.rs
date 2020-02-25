/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::collections::HashSet;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

use atomic_counter::AtomicCounter;
use byteorder::{BigEndian, WriteBytesExt};
use log::{debug, error, info};
use protobuf::{CodedOutputStream, Message, RepeatedField};

use shared::protos::consensus::*;
use state::{ConsensusMachine, MasterMachine, State};
use util;

pub trait ConsensusRequestHandler {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        state: Arc<RwLock<State>>,
    );
}

fn write_single_response(
    message_id: &String,
    single_response: SingleConsensusResponse,
    stream: &mut TcpStream,
) {
    let size = single_response.compute_size();
    if let Ok(_) = stream.write_u32::<BigEndian>(size) {
        let mut output_stream = CodedOutputStream::new(stream);

        match single_response.write_to(&mut output_stream) {
            Ok(_) => {
                (if let Err(e) = output_stream.flush() {
                    error!(
                        "{} || Could not flush stream! Error: {} ",
                        &message_id,
                        e.to_string()
                    );
                })
            }
            Err(e) => {
                error!(
                    "{} || Could not write to stream! Error: {}",
                    &message_id,
                    e.to_string()
                );
            }
        };
    } else {
        error!(
            "{} || Could not write the size of protobuf message to stream!",
            &message_id
        );
    }
}

fn create_heartbeat_response(
    consensus: &Vec<ConsensusMachine>,
    master: &Vec<MasterMachine>,
) -> HeartbeatResponse {
    let mut response = HeartbeatResponse::new();

    let con: Vec<Consensus> = consensus.iter().map(|x| x.to_proto()).collect();

    let mas: Vec<Master> = master.iter().map(|x| x.to_proto()).collect();

    response.set_consensuses(RepeatedField::from_vec(con));
    response.set_masters(RepeatedField::from(mas));

    return response;
}

impl ConsensusRequestHandler for LeaderConnectionRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        state: Arc<RwLock<State>>,
    ) {
        info!(
            "{} || Processing message as a LeaderConnectionRequest",
            &message_id
        );
        let mut response = SingleConsensusResponse::new();

        let mut write_state = state.write().unwrap();
        if write_state.leader.is_none() {
            // I must be the leader
            let val = &write_state.consensus_counter.inc();
            info!("{} || Assign new instance ID: {} ", &message_id, val);

            // Create a consensus machine for it
            let ip_addr = stream.local_addr().unwrap().ip().to_string();
            write_state.consensuses.push(ConsensusMachine {
                id: *val as i32,
                ip: ip_addr,
                port: self.get_port(),
            });

            let mut leader_response = LeaderConnectionResponse::new();
            leader_response.set_consensus_id(*val as i32);
            leader_response.set_heartbeat_response(create_heartbeat_response(
                &write_state.consensuses,
                &write_state.masters,
            ));
            response.set_leader_connection_response(leader_response);
        } else {
            error!(
                "{} || Attempted to connect to consensus that isn't a leader!",
                &message_id
            );
            response.set_not_leader_response(NotLeaderResponse::new());
        }

        write_single_response(&message_id, response, stream);
    }
}

impl ConsensusRequestHandler for HeartbeatRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        state: Arc<RwLock<State>>,
    ) {
        info!(
            "{} || Processing message as a HeartbeatRequest",
            &message_id
        );

        let mut response = SingleConsensusResponse::new();

        let mut write_state = state.write().unwrap();

        let con_set_1: HashSet<i32> = write_state.consensuses.iter().map(|x| x.id).collect();
        let con_set_2: HashSet<i32> = self.take_consensuses().into_iter().map(|x| x.id).collect();
        let con_intersect: HashSet<i32> = con_set_1
            .intersection(&con_set_2)
            .map(|x| x.clone())
            .collect();
        write_state
            .consensuses
            .retain(|x| con_intersect.contains(&x.id));

        // Need to add the sender as a field
        // Need to ensure that the fields sent in the message are updated!

        let mas_set_1: HashSet<i32> = write_state.masters.iter().map(|x| x.id).collect();
        let mas_set_2: HashSet<i32> = self.take_masters().into_iter().map(|x| x.id).collect();
        let mast_intersect: HashSet<i32> = mas_set_1
            .intersection(&mas_set_2)
            .map(|x| x.clone())
            .collect();
        write_state
            .masters
            .retain(|x| mast_intersect.contains(&x.id));

        response.set_heartbeat_response(create_heartbeat_response(
            &write_state.consensuses,
            &write_state.masters,
        ));

        write_single_response(&message_id, response, stream);
    }
}

impl ConsensusRequestHandler for ConflictingActionRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        state: Arc<RwLock<State>>,
    ) {
        info!(
            "{} || Processing message as a ConflictingActionRequest",
            &message_id
        );
        let mut response = SingleConsensusResponse::new();

        let write_state = state.read().unwrap();
        if write_state.leader.is_none() {
            // I must be the leader
            let val = &write_state.conflicting_counter.inc();
            info!("{} || Sending ID: {} ", &message_id, val);
            debug!("H1");
            let mut conflicting_response = ConflictingActionResponse::new();
            debug!("H2");
            conflicting_response.set_id(*val as u32);
            debug!("H3");

            response.set_conflicting_action_response(conflicting_response);
            debug!("H4");
        } else {
            error!(
                "{} || Attempted to connect to consensus that isn't a leader!",
                &message_id
            );
            response.set_not_leader_response(NotLeaderResponse::new());
        }

        write_single_response(&message_id, response, stream);
    }
}

impl ConsensusRequestHandler for UniqueIdRequest {
    fn handle_message(
        &mut self,
        message_id: &String,
        stream: &mut TcpStream,
        state: Arc<RwLock<State>>,
    ) {
        info!("{} || Processing message as a UniqueIdRequest", &message_id);
        let mut response = SingleConsensusResponse::new();

        let mut writable_state = state.write().unwrap();
        if writable_state.leader.is_none() {
            // I must be the leader

            let val = &writable_state.conflicting_counter.inc();
            info!("{} || Sending ID: {} ", &message_id, val);
            let id = util::unique_id(&writable_state.unique_id_source);
            writable_state.unique_id_source.insert(id.clone());

            let mut unique_id_response = UniqueIdResponse::new();
            unique_id_response.set_id(id);

            response.set_unique_id_response(unique_id_response);
        } else {
            error!(
                "{} || Attempted to connect to consensus that isn't a leader!",
                &message_id
            );
            response.set_not_leader_response(NotLeaderResponse::new());
        }

        write_single_response(&message_id, response, stream);
    }
}
