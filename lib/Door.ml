(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

(* open Core.Std *)
open Marshal
open Parli_core_proto.Job_types
open Parli_core_proto.Status_types
open Parli_core_proto.Connection_types

(* Types *)

module Workload =
struct
  exception LastJobMustBeSingleOut

  type jobType = SingleInVariableOut | SingleInSingleOut | VariableInSingleOut
  type job = {
    jobType: jobType;
    functionName: string
  }
  type 'a workload = {
    input: 'a ;
    jobs : job list ;
  }
  let input (x: 'a) = { 
    input = x ;
    jobs = [] ;
  }
  let singleInVariableOut wl name = {
    input = wl.input;
    jobs = {
      jobType = SingleInVariableOut ; 
      functionName = name ;
    }::wl.jobs ;
  }

  let singleInsingleOut wl name = {
    input = wl.input;
    jobs = {
      jobType = SingleInSingleOut ; 
      functionName = name ;
    }::wl.jobs ;
  }

  let variableInSingleOut wl name = {
    input = wl.input;
    jobs = {
      jobType = VariableInSingleOut ; 
      functionName = name ;
    }::wl.jobs ;
  }

  let validate wl = 
    let tail = List.hd (List.rev (wl.jobs)) in 
    match tail.jobType with
    | SingleInVariableOut -> raise LastJobMustBeSingleOut
    | _ -> wl

  let build wl_in starting_id =
    let wl = validate wl_in in 
    let input_bytes = Marshal.to_bytes(wl.input) ([Compat_32]) in
    let input_job = Parli_core_proto.Job_types.({
        job_id = starting_id;
        action = Input(Parli_core_proto.Job_types.({
            data_loc_in = input_bytes
          })
          )
      }) in
    let build_job job prev_id = 
      let map_type_val = (match job.jobType with
          | SingleInSingleOut -> Single_in_variable_out
          | VariableInSingleOut -> Variable_in_variable_out
          | SingleInVariableOut -> Single_in_variable_out)
      in
      Parli_core_proto.Job_types.({
          job_id = (Int32.succ prev_id);
          action = Map(Parli_core_proto.Job_types.({
              map_type = map_type_val;
              job_id_in = prev_id;
              function_name = job.functionName;
            })
            )
        }) in
    let rec build_jobs acc id = function 
      | [] -> List.rev(acc)
      | h::tail -> build_jobs ((build_job(h) (id))::acc) (Int32.add Int32.one id) (tail) 
    in

    input_job::(build_jobs ([]) (starting_id) (wl.jobs))

end

module Context =
struct
  exception UnconnectedException

  type connection_status = Unconnected
                         | Connected
                         | ConnectedRejection

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

  let connect hn pt auth =
    let response = Connection.send_connection_request hn pt auth in
    match response.connection_accepted with
    | true -> 
      {
        hostname = hn ;
        port = pt;
        connection_status = Connected ;
        user_id = response.user_id ;
        next_job = Int32.one ;
      }
    | false -> 
      {
        hostname = hn ;
        port = pt;
        connection_status = Unconnected ;
        user_id = response.user_id ;
        next_job = Int32.one ;
      }

  let validate_context ctx =
    match ctx.connection_status with 
    | Connected -> ctx
    | _ -> (Util.error_print("Context is unconnected to a cluster"); raise UnconnectedException)  

  let submit (ctx_in: context) (jobs: job list) = 
    let ctx = validate_context ctx_in in 
    let job_count = Int32.succ (Int32.succ (Int32.of_int (List.length jobs))) in
    Util.info_print("Submitting " ^ (string_of_int (Int32.to_int job_count)) ^ " jobs to the cluster");
    let single_request = Job_submission(Parli_core_proto.Job_types.({
        user_id = ctx.user_id;
        jobs = jobs;
      })
      ) in
    let running_jobs_list = List.map (fun x -> {job_id = x ; status = Queued}) (Util.range(ctx.next_job) (Int32.add job_count ctx.next_job)) in
    let single_response = Connection.send_single_request(single_request) (ctx.hostname) (ctx.port) in 
    match single_response with
    | Job_submission_response(response) -> (
        if response.job_accepted then ({
            hostname = ctx.hostname ;
            port = ctx.port;
            connection_status = ctx.connection_status ;
            user_id = ctx.user_id ;
            next_job = Int32.add job_count ctx.next_job ;
          }, Some(running_jobs_list))
        else (ctx, None)
      )
    | _ -> (Util.error_print("Recieved a response from server not of type job_submission_response"); (ctx, None))

  let job_status ctx_in jobs =
    let ctx = validate_context ctx_in in
    let single_request = Job_status_request(Parli_core_proto.Job_types.({
        job_ids = List.map (fun x -> x.job_id) jobs
      })
      ) in
    let convert_status_from_proto (status:job_status_status) : status =
      match status with
      | Queued  -> Queued
      | Waiting -> Waiting
      | Running -> Running
      | Completed -> Completed
      | Errored -> Errored
      | Cancelled -> Cancelled
    in
    let proto_to_running_job (proto) =
      {
        job_id = proto.job_id ;
        status = proto.status ;
      } in
    let single_response = Connection.send_single_request(single_request) (ctx.hostname) (ctx.port) in 
    match single_response with
    | Job_status_response(response) -> (
        (ctx, Some(List.map proto_to_running_job response.job_status))
      )
    | _ -> (Util.error_print("Recieved a response from server not of type Job_status_response"); (ctx, None))

  let output ctx_in job_id = 
    let ctx = validate_context ctx_in in


end
