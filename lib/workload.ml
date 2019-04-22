(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

open Marshal
open Datapack
open Parliament_proto.Job_types

(* TYPES *)
exception LastJobMustBeSingleOutput
exception MaxOneInputValue
exception IncorrectFormulationOfStages
exception MustHaveAtLeastOneJob

exception OnlySingleInputAtStage

type job = SingleInMultiOut of (datapack -> datapack) 
         | SingleInSingleOut of (datapack -> datapack) 
         | MultiInSingleOut of (datapack -> datapack) 

type workload = {
  input: datapack ;
  job_list : job list ;
}
let input x = { 
  input = x ;
  job_list = [] ;
}

(* FUNCTIONS *)

let add wl job = {
  input = wl.input;
  job_list = job::wl.job_list ;
}

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
    | 1,MultiInSingleOut(_)::tail -> check (acc-1) (tail)
    | _,MultiInSingleOut(_)::_ -> raise IncorrectFormulationOfStages
    | 0,SingleInMultiOut(_)::tail -> check (acc+1) tail
    | _,SingleInMultiOut(_)::_ -> raise IncorrectFormulationOfStages
  in
  match length wl.input with
    0 -> check 0 jobs
  | 1 -> check 0 jobs
  | _ -> check 1 jobs 

let validate wl =
  branch_validate wl;
  if (List.length wl.job_list = 0) then 
    raise MustHaveAtLeastOneJob
  else
    ()

let build wl starting_id =
  validate wl;
  let input_job = Parliament_proto.Job_types.({
      job_id = starting_id;
      action = Input(Parliament_proto.Job_types.({
          data_loc_in = get_direct wl.input
        })
        )
    }) in
  let build_job job prev_id = 
    let map_type_val, function_closure = (match job with
          SingleInSingleOut(closure) -> Single_in_single_out, closure
        | MultiInSingleOut(closure) -> Multi_in_single_out, closure
        | SingleInMultiOut(closure) -> Single_in_multi_out, closure)
    in
    let closure = Marshal.to_bytes function_closure [Compat_32; Closures] in
    Parliament_proto.Job_types.({
        job_id = (Int32.succ prev_id);
        action = Map(Parliament_proto.Job_types.({
            map_type = map_type_val;
            job_id_in = prev_id;
            function_closure = closure;
          })
          )
      }) in
  let ending_id = Int32.pred (Int32.add starting_id (Int32.of_int (List.length wl.job_list))) in
  let rec build_jobs acc id = function 
    | [] -> acc
    | h::tail -> build_jobs ((build_job(h) (id))::acc) (Int32.pred id) (tail) 
  in

  input_job::(build_jobs ([]) (ending_id) (wl.job_list))

