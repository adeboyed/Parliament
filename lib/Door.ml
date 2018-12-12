(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

(* open Core.Std *)
open Marshal
open Datapack
open Parli_core_proto.Job_types
open Parli_core_proto.Status_types
open Parli_core_proto.Connection_types

(* Types *)

module Workload =
struct
  exception MustHaveSingleOuptput
  exception FirstJobMustBeSingleIn
  exception InputMustBeSingleIn
  exception IncorrectFormulationOfStages

  exception OnlySingleInputAtStage

  type job = SingleInVariableOut of (datapack -> datapack) 
           | SingleInSingleOut of (datapack -> datapack) 
           | VariableInSingleOut of (datapack -> datapack) 

  type workload = {
    input: datapack ;
    job_list : job list ;
  }
  let input x = { 
    input = x ;
    job_list = [] ;
  }

  let stage_validate prev next =
    match (prev, next) with
    | (VariableInSingleOut(_), VariableInSingleOut(_)) -> raise OnlySingleInputAtStage
    | _ -> ()

  let add wl job =
    let rtn_value = {
      input = wl.input;
      job_list = job::wl.job_list ;
    } in
    match wl.job_list with 
    | [] -> rtn_value
    | hd::_ ->  stage_validate hd job; rtn_value

  let add_all wl jobs = 
    let wl_ref = ref wl in 
    List.iter (fun x -> (wl_ref:= add !wl_ref x)) jobs;
    !wl_ref

  let branch_validate wl = 
    let jobs = List.rev wl.job_list in 
    let rec check acc jobs =
      match (acc, jobs) with
        _, [] -> ()
      | _,SingleInSingleOut(_)::tail -> check acc tail
      | 1,VariableInSingleOut(_)::tail -> check (acc-1) (tail)
      | _,VariableInSingleOut(_)::_ -> raise IncorrectFormulationOfStages
      | 0,SingleInVariableOut(_)::tail -> check (acc+1) tail
      | _,SingleInVariableOut(_)::_ -> raise IncorrectFormulationOfStages
    in
    check 0 jobs

  let validate wl =
    branch_validate wl;
    let hd = List.hd(List.rev(wl.job_list)) in
    let tail = List.hd (List.rev (wl.job_list)) in
    match (length wl.input, hd, tail) with
      (_, _, SingleInVariableOut(_)) -> raise MustHaveSingleOuptput
    | (_,VariableInSingleOut(_), _) -> raise FirstJobMustBeSingleIn
    |  (0, _, _) -> wl
    | _ -> raise InputMustBeSingleIn

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
      let map_type_val, function_closure = (match job with
            SingleInSingleOut(closure) -> Single_in_variable_out, closure
          | VariableInSingleOut(closure) -> Variable_in_variable_out, closure
          | SingleInVariableOut(closure) -> Single_in_variable_out, closure)
      in
      let closure = Marshal.to_bytes function_closure [Compat_32; Closures] in
      Parli_core_proto.Job_types.({
          job_id = (Int32.succ prev_id);
          action = Map(Parli_core_proto.Job_types.({
              map_type = map_type_val;
              job_id_in = prev_id;
              function_closure = closure;
            })
            )
        }) in
    let rec build_jobs acc id = function 
      | [] -> acc
      | h::tail -> build_jobs ((build_job(h) (id))::acc) (Int32.add Int32.one id) (tail) 
    in

    input_job::(build_jobs ([]) (starting_id) (wl.job_list))

end

module Context =
struct
  open Workload
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

end