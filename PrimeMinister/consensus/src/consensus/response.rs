/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use state::{ConsensusMachine, MasterMachine, State};
use std::sync::{Arc, RwLock};

use log::info;

use shared::protos::consensus::*;

pub trait ConsensusResponseHandler {
    fn handle_message(&mut self, message_id: &String, state: &Arc<RwLock<State>>) -> bool;
}

fn parse_heartbeat_response(
    response: &mut HeartbeatResponse,
) -> (Vec<ConsensusMachine>, Vec<MasterMachine>) {
    return (
        response
            .take_consensuses()
            .into_iter()
            .map(|x| ConsensusMachine::from_proto(x))
            .collect(),
        response
            .take_masters()
            .into_iter()
            .map(|x| MasterMachine::from_proto(x))
            .collect(),
    );
}

impl ConsensusResponseHandler for LeaderConnectionResponse {
    fn handle_message(&mut self, message_id: &String, state: &Arc<RwLock<State>>) -> bool {
        info!(
            "{} || Processing message as a LeaderConnectionResponse",
            &message_id
        );

        let mut writable_state = state.write().unwrap();
        writable_state.consensus_id = self.get_consensus_id();
        let (consensus, masters) = parse_heartbeat_response(&mut self.take_heartbeat_response());
        writable_state.consensuses = consensus;
        writable_state.masters = masters;
        println!("Request: {:?}", &writable_state.consensuses);

        return true;
    }
}

impl ConsensusResponseHandler for HeartbeatResponse {
    fn handle_message(&mut self, _message_id: &String, state: &Arc<RwLock<State>>) -> bool {
        //info!("{} || Processing message as a HeartbeatResponse", &message_id);
        let mut writable_state = state.write().unwrap();
        let (consensus, masters) = parse_heartbeat_response(self);
        writable_state.consensuses = consensus;
        writable_state.masters = masters;
        return true;
    }
}

impl ConsensusResponseHandler for ConflictingActionResponse {
    fn handle_message(&mut self, message_id: &String, _state: &Arc<RwLock<State>>) -> bool {
        info!(
            "{} || Processing message as a ConflictingActionResponse",
            &message_id
        );
        unimplemented!()
    }
}

impl ConsensusResponseHandler for NotLeaderResponse {
    fn handle_message(&mut self, message_id: &String, _state: &Arc<RwLock<State>>) -> bool {
        info!(
            "{} || Processing message as a NotLeaderResponse",
            &message_id
        );
        unimplemented!()
    }
}

impl ConsensusResponseHandler for UniqueIdResponse {
    fn handle_message(&mut self, message_id: &String, _state: &Arc<RwLock<State>>) -> bool {
        info!(
            "{} || Processing message as a UniqueIdResponse",
            &message_id
        );
        unimplemented!()
    }
}
