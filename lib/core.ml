(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

open Sys
open Door.Context

let init_master : context  = 
  let hn = getenv "PARLIAMENT_HOSTNAME" in
  let pt = int_of_string (getenv "PARLIAMENT_PORT") in
  let user_id = getenv "PARLIAMENT_USER_ID" in
  let ctx_in = {
    hostname = hn;
    port = pt;
    connection_status = Connected ;
    user_id = user_id ;
    next_job = Int32.one ;
  } in
  let ctx = validate (heartbeat ctx_in) in
  ctx


let init_worker = exit 2

let init =
  let internal_init =
    let running_option = getenv "PARLIAMENT_MODE" in
    match running_option with 
    | "Master" -> init_master
    | "Worker" -> init_worker
    | _ -> (Util.error_print("Running option is set to " ^ running_option ^ "! No idea how to deal with"); exit 2)
  in
  try internal_init
  with 
    Not_found -> (Util.error_print("ENV is not setup correctly! Quitting..."); exit 2)
  | NotConnnectedException -> (Util.error_print("Heartbeat request did not succeed!"); exit 2)
