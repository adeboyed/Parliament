(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

open Sys
open Door.Context
open Datapack
open Core
open Parli_core_proto.Connection_types
open Parli_core_proto.Worker_types

let hostname = ref ""
let port = ref 0
let auth = ref ""

exception IncorrectNumberOfOutputs

(* Command line spec *)
let spec =
  let open Command.Spec in
  empty
  +> flag "-h" (required string) ~doc:"STRING Cluster Hostname"
  +> flag "-p" (required int) ~doc:"INTEGER Cluster Port"
  +> flag "-a" (optional_with_default "" string) ~doc:"STRING Cluster Authentication"

let command =
  Command.basic
    ~summary:"Parliament - A distributed general-purpose cluster-computing framework for OCaml"
    spec
    (fun hn pt au () ->
       hostname := hn;
       port := pt;
       auth := au;
    )

let init_master () =
  Command.run ~version:"1.0" ~build_info:"RWO" command;
  try (
    let ctx = connect !hostname !port !auth in
    let ic = open_in Sys.argv.(0) in

    let len = in_channel_length ic in 
    let bytes = Bytes.create len in 
    really_input ic bytes 0 len;

    let single_request = Executable_request(Parli_core_proto.Create_connection_types.({
        user_id = !ctx.user_id;
        executable = bytes;
      })
      ) in
    let single_response = Connection.send_single_request !ctx.hostname !ctx.port single_request in
    match single_response with
      Connection_response({request_accepted = true}) -> ctx
    |_ -> (Util.error_print("Could not submit! Quitting..."); exit 2)
  )
  with Connection.ConnectionError(e) -> (Util.error_print(e); exit 2)


let validate_output map_type datapack = 
  match (map_type, length datapack) with
  | (Single_in_single_out, 1) -> ()
  | (Variable_in_single_out, 1) -> ()
  | (Single_in_variable_out, _) -> ()
  | _ -> raise IncorrectNumberOfOutputs

let init_worker () = 
  try (
    let bytes_in = 
      let len = in_channel_length stdin in 
      let bytes = Bytes.create len in 
      really_input stdin bytes 0 len; 
      bytes 
    in
    let file_out = getenv "PARLIAMENT_OUTPUT" in
    let worker_input = Parli_core_proto.Worker_pb.decode_worker_input(Pbrt.Decoder.of_bytes bytes_in) in
    let datapack_in : datapack = Marshal.from_bytes worker_input.datapack 0 in 
    let job_func : (datapack -> datapack) = Marshal.from_bytes worker_input.function_closure 0 in
    let datapack_out = job_func datapack_in in
    validate_output worker_input.map_type datapack_out;
    let datapack_out_list = List.map (fun x -> Datapack.single x) (Array.to_list datapack_out.data) in
    let worker_output = Parli_core_proto.Worker_types.({
        datapacks = List.map (fun x -> Marshal.to_bytes x [Compat_32]) datapack_out_list
      }) in
    let encoder = Pbrt.Encoder.create () in
    Parli_core_proto.Worker_pb.encode_worker_output worker_output encoder;
    let oc = open_out file_out in
    output_bytes oc (Pbrt.Encoder.to_bytes encoder);
    close_out oc;
    exit 0
  )
  with Not_found -> (Util.error_print("Please check you have initialised the correct ENV variables"); exit 3)

let init () =
  let internal_init () =
    let running_option = getenv "PARLIAMENT_MODE" in
    if Core.Std.String.contains running_option 'W' then 
      init_worker()
    else
      init_master()
  in
  try internal_init()
  with 
    Not_found -> init_master()
  | NotConnnectedException -> (Util.error_print("Application was disconnected during context intialisation!"); exit 2)

