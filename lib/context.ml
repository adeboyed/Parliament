(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

open Workload
open Parli_core_proto.Job_types
open Parli_core_proto.Status_types
open Parli_core_proto.Connection_types

(* TYPES *)
exception NotConnnectedException

type connection_status = Unconnected
                       | Connected
                       | Disconnected

type context = {
  hostname: string ;
  port: int ;
  connection_status: connection_status ;
  user_id: string ;
  next_job : int32 ;
}

type status =
  | Queued 
  | Waiting 
  | Running 
  | Completed 
  | Errored 
  | Cancelled

type running_job = {
  job_id : int32 ;
  status : status
}

(* FUNCTIONS *)

let connect hn pt auth =
  let response = Connection.send_connection_request hn pt auth in
  match response.connection_accepted with
    true -> 
    ref {
      hostname = hn ;
      port = pt;
      connection_status = Connected ;
      user_id = response.user_id ;
      next_job = Int32.one ;
    }
  | false -> 
    ref {
      hostname = hn ;
      port = pt;
      connection_status = Unconnected ;
      user_id = response.user_id ;
      next_job = Int32.one ;
    }

let validate ctx =
  match !ctx.connection_status with 
    Connected -> ()
  | _ -> (Util.error_print("Context is unconnected to a cluster"); raise NotConnnectedException)  

let heartbeat ctx =
  validate ctx;
  let single_request = Connection_request(Parli_core_proto.Connection_types.({
      user_id = !ctx.user_id ;
      action = Heartbeat ;
    })
    ) in
  let single_response = Connection.send_single_request !ctx.hostname !ctx.port single_request in 
  match single_response with
    Connection_response(response) -> (
      if (response.request_accepted) then
        true
      else
        (ctx := {
            hostname = !ctx.hostname ;
            port = !ctx.port ;
            connection_status = Disconnected ;
            user_id = "" ;
            next_job = Int32.one ;
          }; false)
    )
  | _ -> (Util.error_print("Recieved a response from server not of type ConnectionResponse"); false)

let submit ctx workload = 
  validate ctx;
  let job_count = Int32.succ (Int32.succ (Int32.of_int (List.length workload.job_list))) in
  Util.info_print("Submitting " ^ (string_of_int (Int32.to_int job_count)) ^ " jobs to the cluster");
  let jobs = Workload.build workload !ctx.next_job in
  let single_request = Job_submission(Parli_core_proto.Job_types.({
      user_id = !ctx.user_id;
      jobs = jobs;
    })
    ) in
  let running_jobs_list = List.map (fun x -> {job_id = x ; status = Queued}) (Util.range(!ctx.next_job) (Int32.add job_count !ctx.next_job)) in
  let single_response = Connection.send_single_request !ctx.hostname !ctx.port single_request in 
  match single_response with
    Job_submission_response(response) -> (
      if response.job_accepted then (
        ctx:= {
          hostname = !ctx.hostname ;
          port = !ctx.port;
          connection_status = Connected ;
          user_id = !ctx.user_id ;
          next_job = Int32.add job_count !ctx.next_job ;
        }; Some(running_jobs_list))
      else None
    )
  | _ -> (Util.error_print("Recieved a response from server not of type JobSubmissionResponse"); None)

let job_status ctx (jobs:running_job list) =
  validate ctx;
  let single_request = Job_status_request({
      job_ids = List.map (fun (x:running_job) -> x.job_id) jobs
    })
  in
  let convert_status_from_proto (status:job_status_status) : status =
    match status with
    | Queued  -> Queued
    | Waiting -> Waiting
    | Running -> Running
    | Completed -> Completed
    | Errored -> Errored
    | Cancelled -> Cancelled
  in
  let proto_to_running_job (proto: job_status) =
    {
      job_id = proto.job_id ;
      status = convert_status_from_proto proto.status ;
    } in
  let single_response = Connection.send_single_request !ctx.hostname !ctx.port single_request in 
  match single_response with
    Job_status_response(response) -> Some(List.map proto_to_running_job response.job_status)
  | _ -> (Util.error_print("Recieved a response from server not of type Job_status_response"); None)

let rec all_completed = function
  | [] -> true
  | h::tail -> (match h.status with
        Completed -> all_completed tail
      | _ -> false)

let rec wait_until_output ctx (jobs:running_job list) =
  validate ctx;
  let status_option = job_status ctx jobs in
  let status = match status_option with
    | Some(x) -> x
    | None -> raise NotConnnectedException in
  match all_completed status with
    true -> ()
  | false -> wait_until_output ctx jobs

let output ctx job_id = 
  validate ctx;
  let single_request = Data_retrieval_request(Parli_core_proto.Data_types.({
      job_id = job_id;
    })
    ) in
  let single_response = Connection.send_single_request !ctx.hostname !ctx.port single_request in 
  match single_response with
    Data_retrieval_response(response) -> Some(Datapack.single(response.bytes))
  | _ -> (Util.error_print("Recieved a response from server not of type Data_retrieval_response"); None)

let output ctx jobs = output ctx ((List.hd (List.rev jobs)).job_id)


(* TESTS *)