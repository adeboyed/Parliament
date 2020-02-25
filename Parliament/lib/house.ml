(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
 *)


open Sys
open Door.Context
open Datapack

open Parliament_proto.Worker_types

let hostname = ref ""
let port = ref 0
let docker = ref ""
let args = ref ""

exception IncorrectNumberOfOutputs

(* Command line spec *)
let spec =
  let open Core.Command.Spec in
  empty
  +> flag "-h" (required string) ~doc:"STRING Cluster Hostname"
  +> flag "-p" (required int) ~doc:"INTEGER Cluster Port"
  +> flag "-a" (optional_with_default "" string) ~doc:"STRING Program arguments"
  +> flag "-d" (optional_with_default "" string) ~doc:"STRING Executable docker container"

let command =
  Core.Command.basic
    ~summary:"Parliament - A distributed general-purpose cluster-computing framework for OCaml"
    spec
    (fun hn pt au doc () ->
       hostname := hn;
       port := pt;
       args := au;
       docker := doc;
    )

let init_master () =
  Core.Command.run ~version:"1.0" ~build_info:"RWO" command;
  try (
    ((connect !hostname !port !docker), !args)
  )
  with Connection.ConnectionError(e) -> (Util.error_print(e); exit 2)


let validate_output map_type datapack = 
  match (map_type, (Datapack.length datapack)) with
  | (Single_in_single_out,1) -> ()
  | (Multi_in_single_out, 1) -> ()
  | (Single_in_multi_out, _) -> ()
  | _ -> raise IncorrectNumberOfOutputs

let init_worker () = 
  try (
    let worker_hostname = getenv "PARLIAMENT_HOST" in
    let worker_port = int_of_string (getenv "PARLIAMENT_PORT") in
    Util.info_print ("Attempting to connect to worker @ " ^ worker_hostname ^ ":" ^ (string_of_int worker_port)) ;
    let worker_input = (Connection.send_worker_request worker_hostname worker_port Input_request) in
    match worker_input with
      Input_response(input_data) -> (
        let datapack_in : datapack = create_direct input_data.datapacks in
        Util.info_print ("No of inputs: " ^ (string_of_int (Array.length datapack_in.data)) ); 
        let job_func : (datapack -> datapack) = Marshal.from_bytes input_data.function_closure 0 in
        let datapack_out = job_func datapack_in in
        validate_output input_data.map_type datapack_out;
        Util.info_print ("No of outputs: " ^ (string_of_int (Array.length datapack_out.data)) ); 
        let worker_output = Parliament_proto.Worker_types.({
            datapacks = get_direct datapack_out
          }) in
        ignore(Connection.send_worker_request worker_hostname worker_port (Output_request(worker_output)));
        exit 0 
      )
    | _ -> Util.error_print "Recieved an incorrect response from server!"; exit 201
  )
  with Not_found -> (Util.error_print "Please check you have initialised the correct ENV variables"; exit 201)

let init () =
  let internal_init () =
    let running_option = getenv "PARLIAMENT_MODE" in
    if Core.String.contains running_option 'W' then 
      init_worker()
    else
      init_master()
  in
  try internal_init()
  with 
    Not_found -> init_master()
  | NotConnnectedException -> (Util.error_print("Application was disconnected during context intialisation!"); exit 200)

