/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::collections::HashSet;

use atomic_counter::ConsistentCounter;
use shared::protos::consensus::{Consensus, Master};

#[derive(Clone, Debug)]
pub struct ConsensusMachine {
    pub id: i32,
    pub ip: String,
    pub port: i32,
}

impl ConsensusMachine {
    pub fn to_proto(&self) -> Consensus {
        let mut output = Consensus::new();
        output.set_ip(self.ip.clone());
        output.set_id(self.id.clone());
        output.set_port(self.port.clone());
        return output;
    }

    pub fn from_proto(mut cons: Consensus) -> ConsensusMachine {
        return ConsensusMachine {
            id: cons.get_id(),
            ip: cons.take_ip(),
            port: cons.get_port(),
        };
    }
}

#[derive(Clone, Debug)]
pub struct SelfMachine {
    pub ip: String,
    pub con_port: i32,
    pub worker_port: i32,
    pub user_port: i32,
}

impl SelfMachine {
    pub fn to_consensus(&self) -> ConsensusMachine {
        return ConsensusMachine {
            id: 1,
            ip: self.ip.clone(),
            port: self.con_port.clone(),
        };
    }
}

#[derive(Clone, Debug)]
pub struct MasterMachine {
    pub id: i32,
    pub ip: String,
    pub worker_port: i32,
    pub user_port: i32,
    pub active: bool,
}

impl MasterMachine {
    pub fn to_proto(&self) -> Master {
        let mut output = Master::new();
        output.set_id(self.id.clone());
        output.set_ip(self.ip.clone());
        output.set_user_port(self.user_port.clone());
        output.set_worker_port(self.worker_port.clone());
        output.set_active(self.active.clone());
        return output;
    }

    pub fn from_proto(mut mas: Master) -> MasterMachine {
        return MasterMachine {
            id: mas.get_id(),
            ip: mas.take_ip(),
            worker_port: mas.get_worker_port(),
            user_port: mas.get_user_port(),
            active: mas.get_active(),
        };
    }
}

#[derive(Debug)]
pub struct State {
    pub consensus_id: i32,
    pub this: SelfMachine,
    pub leader: Option<ConsensusMachine>,
    pub consensuses: Vec<ConsensusMachine>,
    pub masters: Vec<MasterMachine>,
    pub conflicting_counter: ConsistentCounter,
    pub consensus_counter: ConsistentCounter,
    pub unique_id_source: HashSet<String>,
}

impl Default for State {
    fn default() -> State {
        return State {
            consensus_id: 1,
            this: SelfMachine {
                ip: "0.0.0.0".to_string(),
                con_port: 3060,
                worker_port: 3061,
                user_port: 3062,
            },
            leader: None,
            consensuses: Vec::new(),
            masters: Vec::new(),
            conflicting_counter: ConsistentCounter::new(1),
            consensus_counter: ConsistentCounter::new(2),
            unique_id_source: HashSet::new(),
        };
    }
}
