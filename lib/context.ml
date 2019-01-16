(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

open Workload
open Parliament_proto.Job_types
open Parliament_proto.Status_types
open Parliament_proto.Connection_types

(* TYPES *)
exception NotConnnectedException
exception JobErroredException
exception JobSubmissionException
exception InternalServerError

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
    Blocked
  | Queued 
  | Running 
  | Completed 
  | Halted
  | Cancelled

type running_job = {
  job_id : int32 ;
  status : status
}

(* FUNCTIONS *)

(* let validate_docker_name docker =
  let r = Str.regexp {|[a-zA-Z0-9]*\/[a-zA-Z0-9]*:\(latest\|[0-9.]+\)|} in
  if (String.length docker > 0) && (Str.string_match r docker 0) then
    docker
  else
    "" *)

let connect hn pt docker =
  let single_request = Create_connection_request(Parliament_proto.Create_connection_types.({ 
      authentication = "";
      docker_name = docker;
    })) in
  let single_response = Connection.send_single_request hn pt single_request in
  match single_response with
    Create_connection_response(response) -> (
      match response.connection_accepted with
        true -> Util.info_print("Connected to server as user " ^ response.user_id);
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
          user_id = "" ;
          next_job = Int32.one ;
        }
    )
  | Server_message({action = Internal_server_error }) -> (
      Util.error_print("Recieved an internal server error!");
      raise InternalServerError
    )
  | _ -> (Util.error_print("Recieved a response from server not of type ConnectionResponse"); ref {
      hostname = hn ;
      port = pt;
      connection_status = Unconnected ;
      user_id = "" ;
      next_job = Int32.one ;
    })

let validate ctx =
  match !ctx.connection_status with 
    Connected -> ()
  | _ -> (Util.error_print("Context is unconnected to a cluster"); raise NotConnnectedException)  

let heartbeat ctx =
  validate ctx;
  let single_request = Connection_request(Parliament_proto.Connection_types.({
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
  | Server_message({action = Internal_server_error }) -> (
      Util.error_print("Recieved an internal server error!");
      raise InternalServerError
    )
  | _ -> (Util.error_print("Recieved a response from server not of type ConnectionResponse"); false)

let submit ctx workload = 
  validate ctx;
  let job_count = Int32.of_int (List.length workload.job_list)in
  Util.info_print("Submitting " ^ (Int32.to_string job_count) ^ " jobs to the cluster");

  let jobs = Workload.build workload !ctx.next_job in
  let single_request = Job_submission(Parliament_proto.Job_types.({
      user_id = !ctx.user_id;
      jobs = jobs;
    })
    ) in
  let running_jobs_list = List.tl (List.map (fun x -> {job_id = x ; status = Queued}) (Util.range(!ctx.next_job) (Int32.add job_count !ctx.next_job))) in
  let single_response = Connection.send_single_request !ctx.hostname !ctx.port single_request in 
  match single_response with
    Job_submission_response(response) -> (
      if response.job_accepted then (
        ctx := {
          hostname = !ctx.hostname ;
          port = !ctx.port;
          connection_status = Connected ;
          user_id = !ctx.user_id ;
          next_job = Int32.succ (Int32.add job_count !ctx.next_job) ;
        }; Some(running_jobs_list))
      else None
    )
  | Server_message({action = Internal_server_error }) -> (
      Util.error_print("Recieved an internal server error!");
      raise InternalServerError
    )
  |_ -> (Util.error_print("Recieved a response from server not of type JobSubmissionResponse"); None)

let job_status ctx (jobs:running_job list) =
  validate ctx;
  let single_request = Job_status_request({
      user_id = !ctx.user_id;
      job_ids = List.map (fun (x:running_job) -> x.job_id) jobs
    })
  in
  let convert_status_from_proto (status:user_job_status_status) : status =
    match status with
      Blocked -> Blocked
    | Queued  -> Queued
    | Running -> Running
    | Completed -> Completed
    | Halted -> Halted
    | Cancelled -> Cancelled
  in
  let proto_to_running_job (proto: user_job_status) =
    {
      job_id = proto.job_id ;
      status = convert_status_from_proto proto.status ;
    } in
  let single_response = Connection.send_single_request !ctx.hostname !ctx.port single_request in 
  match single_response with
    Job_status_response(response) -> Some(List.map proto_to_running_job response.job_statuses)
  | Server_message({action = Internal_server_error }) -> (
      Util.error_print("Recieved an internal server error!");
      raise InternalServerError
    )
  | Server_message({action = Missing_jobs }) -> (
      Util.error_print("Server cannot find all of the jobs, raising exception...");
      raise JobSubmissionException
    )
  | _ -> (Util.error_print("Recieved a response from server not of type Job_status_response"); None)

let rec all_completed = function
  | [] -> true
  | {status = Completed; job_id =  _}::tail -> all_completed tail
  | _ -> false

let rec cancelled_or_halted = function
  | [] -> false
  | {status = Cancelled; job_id =  _}::_ -> true
  | {status = Halted; job_id =  _}::_ -> true
  | _::tail -> cancelled_or_halted tail

let rec wait_until_output ctx (jobs:running_job list) =
  validate ctx;
  let status_option = job_status ctx jobs in
  let status = match status_option with
    | Some(x) -> x
    | None -> raise NotConnnectedException in
  match (all_completed status, cancelled_or_halted status) with
    true, _ -> ()
  | false, true -> raise JobErroredException
  | false, false -> (Util.minisleep 2.0; wait_until_output ctx jobs)

let output ctx job_id = 
  validate ctx;
  let single_request = Data_retrieval_request(Parliament_proto.Data_types.({
      user_id = !ctx.user_id;
      job_id = job_id;
    })
    ) in
  let single_response = Connection.send_single_request !ctx.hostname !ctx.port single_request in 
  match single_response with
    Data_retrieval_response(response) -> Some(Datapack.create_direct response.bytes)
  | Server_message({action = Internal_server_error }) -> (
      Util.error_print("Recieved an internal server error!");
      raise InternalServerError
    )
  | _ -> (Util.error_print("Recieved a response from server not of type Data_retrieval_response"); None)

let output ctx jobs = output ctx ((List.hd (List.rev jobs)).job_id)