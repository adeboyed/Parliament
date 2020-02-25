/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::process::Command;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::thread;
use timebomb::timeout_ms;

extern crate assert_cli;
extern crate timebomb;

#[cfg(test)]
mod integration {
    use super::*;


    fn executable_path(name: &str) -> PathBuf {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        let exe = String::from(name) + std::env::consts::EXE_SUFFIX;
        path.push(exe);
        path
    }

    #[test]
    fn connects_to_correct_ip() {
        let path = executable_path("member_of_parliament");
        let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
        println!("{}", path.to_str().unwrap().to_owned());
        let mut child = Command::new(path.to_str().unwrap().to_owned())
            .arg("--masterport")
            .arg("3000")
            .spawn()
            .expect("failed to execute process");


        println!("listening started, ready to accept");
        timeout_ms(move || {
            for stream in listener.incoming() {
                child.kill();
                assert_eq!(true, true);
                break;
            }
        }, 1000);
    }

    #[test]
    fn submits_a_worker_request() {
        let path = executable_path("member_of_parliament");
        let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
        println!("{}", path.to_str().unwrap().to_owned());
        let mut child = Command::new(path.to_str().unwrap().to_owned())
            .arg("--masterport")
            .arg("3000")
            .spawn()
            .expect("failed to execute process");


        println!("listening started, ready to accept");
        timeout_ms(move || {
            for stream in listener.incoming() {
                child.kill();
                assert_eq!(true, true);
                break;
            }
        }, 1000);
    }



}