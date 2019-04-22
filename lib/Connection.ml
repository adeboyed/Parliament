(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

open Unix

exception ConnectionError of string

(* Helper functions *)
let open_connection (sockaddr:sockaddr) =
  let sock = Unix.socket Unix.PF_INET Unix.SOCK_STREAM 0 
  in try Unix.connect sock sockaddr ;
    (Unix.in_channel_of_descr sock , Unix.out_channel_of_descr sock)
  with exn -> Unix.close sock ; raise exn

let shutdown_connection (ic:in_channel) =
  Unix.shutdown (Unix.descr_of_in_channel ic) Unix.SHUTDOWN_SEND

let send_to_master client_fun (server: string) (port:int) =
  let server_addr =
    try  Unix.inet_addr_of_string server 
    with Failure(_) -> 
    try  (Unix.gethostbyname server).Unix.h_addr_list.(0) 
    with Not_found -> (raise (ConnectionError "Could not find server from hostname"))
  in try
    let sockaddr = Unix.ADDR_INET(server_addr, port) in 
    let ic, oc = open_connection sockaddr in
    let result = client_fun ic oc in
    shutdown_connection ic;
    result
  with Failure(_) -> raise (ConnectionError "Bad port number")
     | Unix_error(e, _, _) -> raise (ConnectionError ("Could not connect to cluster! " ^ (error_message e) ^ "\nPlease check hostname and port again!" ))

let request_response request response ic oc  =
  let encoder = Pbrt.Encoder.create () in
  request encoder;
  (* Util.info_print "Request encoder"; *)
  let bytes_out = Pbrt.Encoder.to_bytes encoder in
  let bytes_len = (Bytes.length bytes_out) in 
  output_binary_int oc bytes_len;
  output_bytes oc bytes_out;
  flush oc;
  Util.debug_print "Flushed output_channel" ;
  let len, bytes = 
    let len = input_binary_int ic in 
    Util.debug_print ("Recieved length of " ^ (string_of_int len) ^ " from socket" );
    let bytes = Bytes.create len in 
    really_input ic bytes 0 len; 
    len, bytes 
  in
  if len > 0 then 
    response (Pbrt.Decoder.of_bytes bytes)
  else 
    raise (ConnectionError "Did not recieve anything back from the server!")

let rec retry_handler (func: unit -> 'a) (count:int) = 
  match count with
  0 -> raise (ConnectionError "Run out of retries")
  | count -> 
    try func ()
    with ConnectionError(_) -> (retry_handler func (count-1))

(* Actual useful functions *)
let send_single_request hostname port request_obj = 
  let request = Parliament_proto.Connection_pb.encode_single_user_request request_obj in 
  let response = Parliament_proto.Connection_pb.decode_single_user_response in
  let func() = send_to_master (request_response request response) (hostname) (port) in
  retry_handler func 3

let send_worker_request hostname port request_obj = 
  let request = Parliament_proto.Worker_pb.encode_single_worker_request request_obj in 
  let response = Parliament_proto.Worker_pb.decode_single_worker_response in
  let func () = send_to_master (request_response request response) (hostname) (port) in 
  retry_handler func 3

