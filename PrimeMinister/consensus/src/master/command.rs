/*
    Parliament - A distributed general-purpose cluster-computing framework for OCaml
    Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
*/

use std::net::TcpStream;

use byteorder::{BigEndian, WriteBytesExt};
use log::error;
use protobuf::{CodedOutputStream, Message};

use shared::protos::intra_cluster::*;
use shared::util;
use state::MasterMachine;

pub fn send_to_worker_port(
    message: &SingleWorkerMessage,
    master: &MasterMachine,
) -> Result<SingleServerMessage, ()> {
    let stream_res = TcpStream::connect(format!("{}:{}", &master.ip, &master.worker_port));
    if stream_res.is_err() {
        error!(
            "Could not connect to master! Error: {}",
            stream_res.unwrap_err().to_string()
        );
        return Err(());
    }
    let mut stream = stream_res.unwrap();

    {
        let size = message.compute_size();
        stream.write_u32::<BigEndian>(size.clone());
        stream.write_u32::<BigEndian>(0);
        let mut output_stream = CodedOutputStream::new(&mut stream);
        message.write_to(&mut output_stream).unwrap();
        output_stream.flush().unwrap();
    }
    let mut message = SingleServerMessage::new();
    if let Ok(_) = util::process_input(&mut stream, &mut message, false) {
        return Ok(message);
    } else {
        return Err(());
    }
}
