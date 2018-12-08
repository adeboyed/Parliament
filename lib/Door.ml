(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

(* open Core.Std *)
open Marshal
open Parli_core_proto.Job_types
open Parli_core_proto.Connection_types
open Mutex

(* Types *)

type connection_status = Unconnected
  | ConnectionPending
  | Connected
  | ConnectedRejection

class parliament_context = 
  object
    val mutable hostname: string = ""
    val mutable port : int = 0
    val mutable connection_status : connection_status = Unconnected
    val mutable user_id : string = ""

    val mutable job_id_counter : int32 = Int32.one;
    val mutable counter_lock : Mutex.t = Mutex.create();

    method connnect(hn:string) (pt:int) (authentication:string) = 
      let response = Connection.send_connection_request(hn) (pt) (authentication) in
      match response.connection_accepted with
          | true -> 
            (hostname <- hn;
            port <- pt;
            connection_status <- Connected;
            user_id <- response.user_id;
            true)
          | false -> 
            (connection_status <- Unconnected;
            false)

    method hostname = hostname
    method port = port
          
    method start_submission =
      (* counter_lock.lock(); *)
      (user_id, job_id_counter)
    
    method stop_submission (new_id) =
      job_id_counter <- new_id;
      (* counter_lock.unlock(); *)
      true
    
  end


module Workload =
  struct
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
    let singleInVariableOut (wl: 'a workload) (name:string) = {
      input = wl.input;
      jobs = {
        jobType = SingleInVariableOut ; 
        functionName = name ;
      }::wl.jobs ;
    }

    let singleInsingleOut (wl: 'a workload) (name:string) = {
      input = wl.input;
      jobs = {
        jobType = SingleInSingleOut ; 
        functionName = name ;
      }::wl.jobs ;
    }

    let variableInSingleOut (wl: 'a workload) (name:string) = {
      input = wl.input;
      jobs = {
        jobType = VariableInSingleOut ; 
        functionName = name ;
      }::wl.jobs ;
    }

    let _build (wl: 'a workload) (starting_id: int32) = 
      let input_bytes = Marshal.to_bytes(wl.input) ([Compat_32]) in
      let input_job = Parli_core_proto.Job_types.({
        job_id = starting_id;
        action = Input(Parli_core_proto.Job_types.({
            data_loc_in = input_bytes
          })
        )
      }) in
      let build_job (job: job) (prev_id: int32) = 
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
      let rec build_jobs (acc) (id) = function 
        | [] -> List.rev(acc)
        | h::tail -> build_jobs (build_job(h)::acc) (id+1) (tail) in
      input_job::(build_jobs([]) (starting_id) (wl.jobs))

    let submit (wl: 'a workload) (ctx: parliament_context) =
      let user_id, counter = ctx#start_submission in
      let jobs = _build(wl) (counter) in
      let single_request = Job_submission(Parli_core_proto.Job_types.({
          user_id = user_id;
          jobs = jobs;
        })
      ) in
      let response = Connection.send_single_request(single_request) (ctx#hostname) (ctx#port) in 
        5

  end



