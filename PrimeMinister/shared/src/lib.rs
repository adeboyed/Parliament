/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

extern crate rand;
extern crate protobuf;
extern crate byteorder;
extern crate log;
extern crate tokio;
extern crate futures;

pub mod util;
pub mod protos;


#[derive(Debug)]
pub struct BoolWrapper {
    value: bool
}


impl BoolWrapper {
    pub fn new(val: bool) -> BoolWrapper {
        return BoolWrapper {
            value: val
        };
    }
    pub fn set_value(&mut self, val: bool) {
        self.value = val;
    }
    pub fn get_value(&self) -> bool {
        return self.value;
    }
}