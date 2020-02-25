(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
 *)

open Unix

open Parliament_proto.Connection_pb
open Parliament_proto.Connection_types

(* Helpful functions *)

let establish_server single_shot server_fun sockaddr =
  let sock = socket PF_INET SOCK_STREAM 0 in
  bind sock sockaddr ;
  listen sock 3;
  if single_shot then
    let (s, _) = accept sock in
    let inchan = in_channel_of_descr s 
      and outchan = out_channel_of_descr s in 
      server_fun inchan outchan ;
      close_in inchan ;
      close_out outchan
  else
  while true do
    let (s, _) = accept sock 
    in match fork() with
           0 -> if fork() <> 0 then exit 0 ; 
                let inchan = in_channel_of_descr s 
                and outchan = out_channel_of_descr s 
                in server_fun inchan outchan ;
                   close_in inchan ;
                   close_out outchan ;
                   exit 0
         | id -> close s; ignore(waitpid [] id)
  done

let get_my_addr () =
  (gethostbyname(gethostname())).h_addr_list.(0)

let standard_server ic oc  =
  let bytes = 
    let len = input_binary_int ic in 
    let bytes = Bytes.create len in 
    really_input ic bytes 0 len; 
    bytes 
  in
  let request = decode_single_user_request (Pbrt.Decoder.of_bytes bytes) in
  match request with
    Create_connection_request(request) -> ()
  | Connection_request(request) -> ()
  | Job_submission(request) -> ()
  | Data_retrieval_request(request) -> ()
  | Job_status_request(request) -> ()
  | Executable_request(request) -> ()

(* Test functions *)

let run_connect_to_correct_server_test port =
  let output_fd, input_fd = pipe() in
  let output_channel = out_channel_of_descr output_fd in 
  let pid = fork () in
  if pid <> 0 then
    input_fd
  else
    let server_details = (ADDR_INET(get_my_addr(), port)) in
    let server_fun _ _ =
      output_value output_channel true;
       in
    establish_server true server_fun server_details;
    Unix.close output_fd;
    exit 0