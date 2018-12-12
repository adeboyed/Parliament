(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

open Unix

exception ConnectionError of string

(* Helper functions *)
let _open_connection (sockaddr:sockaddr) =
  let sock = Unix.socket Unix.PF_INET Unix.SOCK_STREAM 0 
  in try Unix.connect sock sockaddr ;
    (Unix.in_channel_of_descr sock , Unix.out_channel_of_descr sock)
  with exn -> Unix.close sock ; raise exn

let _shutdown_connection (ic:in_channel) =
  Unix.shutdown (Unix.descr_of_in_channel ic) Unix.SHUTDOWN_SEND

let _send_to_master client_fun (server: string) (port:int) =
  let server_addr =
    try  Unix.inet_addr_of_string server 
    with Failure("inet_addr_of_string") -> 
    try  (Unix.gethostbyname server).Unix.h_addr_list.(0) 
    with Not_found -> (raise (ConnectionError "Could not find server from hostname"))
  in try
    let sockaddr = Unix.ADDR_INET(server_addr, port) in 
    let ic,oc = _open_connection sockaddr in
    let result = client_fun ic oc in

    _shutdown_connection ic;
    result
  with Failure("int_of_string") -> raise (ConnectionError "Bad port number")
     | Unix_error(e, _, _) -> raise (ConnectionError ("Could not connect to cluster! " ^ (error_message e) ^ "\nPlease check hostname and port again!" ))

let _request_response request response ic oc  =
  let encoder = Pbrt.Encoder.create () in
  request encoder;
  output_bytes oc (Pbrt.Encoder.to_bytes encoder);
  close_out oc;
  let bytes = 
    let len = in_channel_length ic in 
    let bytes = Bytes.create len in 
    really_input ic bytes 0 len; 
    close_in ic; 
    bytes 
  in 
  response (Pbrt.Decoder.of_bytes bytes)
(* Actual useful functions *)

let send_connection_request hostname port auth =
  let request_obj = Parli_core_proto.Create_connection_types.({ 
      authentication = auth
    }) in
  let request = Parli_core_proto.Create_connection_pb.encode_create_connection_request request_obj in 
  let response = Parli_core_proto.Create_connection_pb.decode_create_connection_response in
  _send_to_master (_request_response request response) (hostname) (port)

let send_single_request hostname port request_obj = 
  let request = Parli_core_proto.Connection_pb.encode_single_request request_obj in 
  let response = Parli_core_proto.Connection_pb.decode_single_response in
  _send_to_master (_request_response request response) (hostname) (port)
