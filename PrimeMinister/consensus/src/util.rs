/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]

    util.rs - Helpful functions used everywhere
*/

use log::warn;

use shared::util::random_alphanum_string;
use state::{ConsensusMachine, MasterMachine, SelfMachine};
use std::collections::HashSet;

pub fn split_and_validate_self_machine(input: String) -> Option<SelfMachine> {
    let mut parts: Vec<&str> = input.split(":").collect();

    if parts.len() != 4 {
        warn!("{} does not have 4 parts!", &input);
        return None;
    }

    let user_port = parts.pop().unwrap();
    let worker_port = parts.pop().unwrap();
    let con_port = parts.pop().unwrap();
    let hostname = parts.pop().unwrap();

    return Some(SelfMachine {
        ip: hostname.to_string(),
        con_port: con_port.parse::<i32>().unwrap(),
        worker_port: worker_port.parse::<i32>().unwrap(),
        user_port: user_port.parse::<i32>().unwrap(),
    });
}

pub fn split_and_validate_consensus_machine(input: String) -> Option<ConsensusMachine> {
    let mut parts: Vec<&str> = input.split(":").collect();

    if parts.len() != 2 {
        warn!("{} does not have 2 parts!", &input);
        return None;
    }

    let port = parts.pop().unwrap();
    let hostname = parts.pop().unwrap();

    return Some(ConsensusMachine {
        id: 1,
        ip: hostname.to_string(),
        port: port.parse::<i32>().unwrap(),
    });
}

pub fn split_and_validate_master_machine(input: String, id: &i32) -> Option<MasterMachine> {
    let mut parts: Vec<&str> = input.split(":").collect();

    if parts.len() != 3 {
        warn!("{} does not have 3 parts!", &input);
        return None;
    }

    let user_port = parts.pop().unwrap();
    let worker_port = parts.pop().unwrap();
    let hostname = parts.pop().unwrap();

    let master = MasterMachine {
        id: id.clone(),
        ip: hostname.to_string(),
        worker_port: worker_port.parse::<i32>().unwrap(),
        user_port: user_port.parse::<i32>().unwrap(),
        active: false,
    };

    println!("Master: {:?}", &master);

    return Some(master);
}

pub fn unique_id(items: &HashSet<String>) -> String {
    let mut user_id = random_alphanum_string(5);

    while items.contains(&user_id) {
        user_id = random_alphanum_string(5);
    }

    return user_id;
}
