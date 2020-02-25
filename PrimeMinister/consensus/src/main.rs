/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

extern crate atomic_counter;
extern crate byteorder;
extern crate chashmap;
extern crate clap;
extern crate crossbeam_channel;
extern crate futures;
extern crate ini;
extern crate log;
extern crate protobuf;
extern crate shared;
extern crate simple_logger;
extern crate tokio_threadpool;

mod consensus;
mod master;
mod state;
mod util;

use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

use clap::{clap_app, ArgMatches};
use crossbeam_channel::unbounded;
use crossbeam_channel::Sender;
use log::{error, info};
use tokio_threadpool::ThreadPool;

use consensus::client::ConsensusUpdate;
use master::command;
use shared::protos::consensus::{Consensus, Master};
use shared::protos::intra_cluster::ConsensusRequest;
use shared::protos::intra_cluster::ConsensusRequest_Action;
use shared::protos::intra_cluster::SingleWorkerMessage;
use state::{ConsensusMachine, MasterMachine, State};

fn load_state(arg: ArgMatches) -> State {
    let mut state = State::default();

    if let Some(export) = arg.value_of("EXPORT") {
        if let Some(this) = util::split_and_validate_self_machine(export.to_string()) {
            state.this = this;
        }
    }

    if arg.is_present("LEADER_TAG") {
        if let Some(masters) = arg.values_of_lossy("MASTERS") {
            let mut id: i32 = 2;
            for x in masters {
                let master = util::split_and_validate_master_machine(x, &id);
                id = id + 1;
                if master.is_none() {
                    panic!("Incorrectly defined master!");
                } else {
                    state.masters.push(master.unwrap());
                }
            }
        } else {
            panic!("No master defined!");
        }
        state.consensuses.push(state.this.to_consensus());
    } else {
        if let Some(leader_str) = arg.value_of("LEADER") {
            if let Some(leader) = util::split_and_validate_consensus_machine(leader_str.to_string())
            {
                state.leader = Some(leader);
            } else {
                panic!("Leader in invalid format!");
            }
        } else {
            panic!("No leader defined!");
        }
    }

    return state;
}

fn heartbeat_messages(sender: Sender<ConsensusUpdate>, state: Arc<RwLock<State>>) {
    loop {
        {
            let readable_state = state.read().unwrap();

            let masters_int = readable_state.masters.clone();

            println!("Consensuses: {:?}", &readable_state.consensuses);
            println!("Masters: {:?}", &readable_state.masters);
            let consensuses_int = readable_state.consensuses.clone();

            let masters: Vec<Master> = masters_int
                .into_iter()
                .map(|x| MasterMachine::to_proto(&x))
                .collect();

            let consensuses: Vec<Consensus> = consensuses_int
                .iter()
                .map(|x| ConsensusMachine::to_proto(x))
                .collect();

            for consensus in consensuses_int.into_iter() {
                if consensus.id != readable_state.consensus_id {
                    let masters = masters.clone();
                    let consensuses = consensuses.clone();
                    let message =
                        ConsensusUpdate::new_heartbeat_message(consensus, consensuses, masters);
                    sender
                        .send(message)
                        .expect("Internal message broker is broken!");
                }
            }
        }

        sleep(Duration::from_millis(2000));
    }
}

fn assign_master(state: &Arc<RwLock<State>>) -> bool {
    let mut writable_state = state.write().unwrap();

    while writable_state.masters.len() > 0 {
        let mut master = writable_state.masters.pop().unwrap();
        master.active = true;
        info!("Attempting to assign master/{} as master", &master.id);

        let mut consensus_request = ConsensusRequest::new();
        consensus_request.set_action(ConsensusRequest_Action::SET_ACTIVE);

        let mut message = SingleWorkerMessage::new();
        message.set_consensus_request(consensus_request);

        let mut retries = 0;
        while retries < 3 {
            match command::send_to_worker_port(&message, &master) {
                Ok(_) => {
                    info!("Assigned to master/{} successfully!", &master.id);
                    writable_state.masters.push(master);
                    return true;
                }
                Err(_) => (),
            }
            retries = retries + 1;
        }
    }
    return false;
}

fn main() {
    //Setup logging
    simple_logger::init().unwrap();

    let matches = clap_app!(myapp =>
        (version: "0.23")
        (author: "[Name REDACTED] [Email REDACTED]")
        (about: "Consensus module ")

        (@arg EXPORT: --export -e +takes_value "[IP:ConPort:WorkerPort:UserPort] of the exposed Consensus TCP socket on. Default: 127.0.0.1:3060:3061:3062")
        (@arg LEADER: --leader -l +takes_value "[IP:ConPort] of the Consensus leader. Ignored if set to leader.")

        (@arg LEADER_TAG: --initial -i "Sets current consensus program as leader")
        (@arg MASTERS: --masters -m +multiple +takes_value "[Host:WorkerPort:UserPort] of masters. Default: empty list. Only valid if leader!")
    );

    let state_out = load_state(matches.get_matches());
    let state = Arc::new(RwLock::new(state_out));

    let (update_sender, update_reciever) = unbounded();

    let threadpool = ThreadPool::new();
    let pool_sender = Arc::new(threadpool.sender().clone());

    consensus::server::start(state.clone()).expect("Could not start consensus server!");

    consensus::client::start(
        state.clone(),
        update_sender.clone(),
        update_reciever.clone(),
    );

    master::worker_server::start(state.clone(), pool_sender.clone(), update_sender.clone())
        .expect("Could not start consensus worker server!");

    master::user_server::start(state.clone(), pool_sender.clone(), update_sender.clone())
        .expect("Could not start the consensus user server");

    let is_leader = {
        let writable_state = &state.write().unwrap();
        let leader_option = &writable_state.leader;
        if let Some(leader) = leader_option {
            // Send connection req
            let message = ConsensusUpdate::new_leader_connection(
                leader.clone(),
                writable_state.this.con_port.clone(),
            );
            update_sender
                .send(message)
                .expect("Internal message broker is broken!");
            false
        } else {
            true
        }
    };

    if is_leader && !assign_master(&state) {
        error!("Could not assign any of the masters as ACTIVE, ending...");
        std::process::exit(2);
    }

    heartbeat_messages(update_sender, state);
}
