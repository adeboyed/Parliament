(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

open Sys
open Door.Context
open Datapack
open Core.In_channel

open Parliament_proto.Worker_types

let hostname = ref ""
let port = ref 0
let auth = ref ""
let docker = ref ""

exception IncorrectNumberOfOutputs
exception IOError of string

(* Command line spec *)
let spec =
  let open Core.Command.Spec in
  empty
  +> flag "-h" (required string) ~doc:"STRING Cluster Hostname"
  +> flag "-p" (required int) ~doc:"INTEGER Cluster Port"
  +> flag "-a" (optional_with_default "" string) ~doc:"STRING Cluster Authentication"
  +> flag "-d" (optional_with_default "" string) ~doc:"STRING Executable docker container"

let command =
  Core.Command.basic
    ~summary:"Parliament - A distributed general-purpose cluster-computing framework for OCaml"
    spec
    (fun hn pt au doc () ->
       hostname := hn;
       port := pt;
       auth := au;
       docker := doc;
    )

let init_master () =
  Core.Command.run ~version:"1.0" ~build_info:"RWO" command;
  try (
    connect !hostname !port !auth !docker
  )
  with Connection.ConnectionError(e) -> (Util.error_print(e); exit 2)


let validate_output map_type datapack = 
  match (map_type, (Datapack.length datapack)) with
  | (Single_in_single_out,1) -> ()
  | (Variable_in_single_out, 1) -> ()
  | (Single_in_variable_out, _) -> ()
  | _ -> raise IncorrectNumberOfOutputs

let load_bytes () =
  match input_binary_int stdin with
        Some(len) -> (
            let bytes = Bytes.create len in 
            match really_input stdin ~buf:bytes ~pos:0 ~len:len with
              Some(_) -> bytes
              | None -> raise (IOError "Couldn't read bytes")
          )
        | None -> raise (IOError "Couldn't read length")

let init_worker () = 
  try (
    let bytes_in = load_bytes() in
    let file_out = getenv "PARLIAMENT_OUTPUT" in
    let worker_input = Parliament_proto.Worker_pb.decode_worker_input(Pbrt.Decoder.of_bytes bytes_in) in
    let datapack_in : datapack = create_direct worker_input.datapacks in
    Util.info_print ("No of inputs: " ^ (string_of_int (Array.length datapack_in.data)) ); 
    let job_func : (datapack -> datapack) = Marshal.from_bytes worker_input.function_closure 0 in
    let datapack_out = job_func datapack_in in
    validate_output worker_input.map_type datapack_out;
    Util.info_print ("No of outputs: " ^ (string_of_int (Array.length datapack_out.data)) ); 
    let worker_output = Parliament_proto.Worker_types.({
        datapacks = get_direct datapack_out
      }) in
    let encoder = Pbrt.Encoder.create () in
    Parliament_proto.Worker_pb.encode_worker_output worker_output encoder;
    let oc = open_out file_out in
    output_bytes oc (Pbrt.Encoder.to_bytes encoder);
    flush oc;
    close_out oc;
    exit 0
  )
  with IOError(e) -> (Util.error_print ("Recieved IO Error: " ^ e) ; exit 202)
  | Not_found -> (Util.error_print "Please check you have initialised the correct ENV variables"; exit 201)

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

