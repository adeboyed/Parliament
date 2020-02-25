/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::net::TcpStream;
use tokio::net::TcpStream as NTcpStream;
use std::io::Read;

use log::error;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use byteorder::{ReadBytesExt, BigEndian};
use protobuf::{Message, ProtobufError, CodedInputStream};
use protobuf::error::WireError;

pub fn random_alphanum_string(length: usize) -> String {
    return thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .collect();
}

pub fn process_input<T: Message>(mut stream: &TcpStream, message: &mut T, consensus_mode: bool) -> Result<(usize), ProtobufError> {
    let size_option = stream.read_u32::<BigEndian>();
    if size_option.is_err() {
        error!("Could not read from stream!");
        return Err(ProtobufError::WireError(WireError::Other));
    }
    let size = size_option.unwrap();

    let id = {
        if consensus_mode {
            let id_option = stream.read_u32::<BigEndian>();
            if id_option.is_err() {
                error!("Could not read from stream!");
                return Err(ProtobufError::WireError(WireError::Other));
            }
            id_option.unwrap() as usize
        } else {
            0
        }
    };

    let mut buffer = vec![0u8; size as usize];
    if let Ok(_) = stream.read_exact(&mut buffer) {
        let mut cis = CodedInputStream::from_bytes(&buffer);
        return match message.merge_from(&mut cis) {
            Ok(()) => Ok(id),
            Err(e) => Err(e)
        };
    }else {
        return Err(ProtobufError::WireError(WireError::Other));
    }
}

pub fn process_input_non_blocking<T: Message>(mut stream: &NTcpStream, message: &mut T, consensus_mode: bool) -> Result<(usize), ProtobufError> {
    let size_option = stream.read_u32::<BigEndian>();
    if size_option.is_err() {
        error!("Could not read from stream!");
        return Err(ProtobufError::WireError(WireError::Other));
    }
    let size = size_option.unwrap();

    let id = {
        if consensus_mode {
            let id_option = stream.read_u32::<BigEndian>();
            if id_option.is_err() {
                error!("Could not read from stream!");
                return Err(ProtobufError::WireError(WireError::Other));
            }
            id_option.unwrap() as usize
        } else {
            0
        }
    };

    let mut buffer = vec![0u8; size as usize];
    if let Ok(_) = stream.read_exact(&mut buffer) {
        let mut cis = CodedInputStream::from_bytes(&buffer);
        return match message.merge_from(&mut cis) {
            Ok(()) => Ok(id),
            Err(e) => Err(e)
        };
    }else {
        return Err(ProtobufError::WireError(WireError::Other));
    }
}
