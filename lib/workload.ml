(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

open Marshal
open Datapack
open Parli_core_proto.Job_types

(* TYPES *)
exception LastJobMustBeSingleOutput
exception MaxOneInputValue
exception IncorrectFormulationOfStages
exception MustHaveAtLeastOneJob

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

(* FUNCTIONS *)

let stage_validate prev next =
  match (prev, next) with
  | (VariableInSingleOut(_), VariableInSingleOut(_)) -> raise OnlySingleInputAtStage
  | _ -> ()

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
    | 1,VariableInSingleOut(_)::tail -> check (acc-1) (tail)
    | _,VariableInSingleOut(_)::_ -> raise IncorrectFormulationOfStages
    | 0,SingleInVariableOut(_)::tail -> check (acc+1) tail
    | _,SingleInVariableOut(_)::_ -> raise IncorrectFormulationOfStages
  in
  check 0 jobs

let validate wl =
  branch_validate wl;
  if (List.length wl.job_list = 0) then 
    raise MustHaveAtLeastOneJob
  else
    match (length wl.input, List.hd (wl.job_list) ) with
    |  (_, SingleInVariableOut(_)) -> raise LastJobMustBeSingleOutput
    |  (0, _) -> () (* We allow no input for first job*)
    |  (1, _) -> () (* ..or one input value *)
    | _ -> raise MaxOneInputValue

let build wl starting_id =
  validate wl;
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
        | VariableInSingleOut(closure) -> Variable_in_single_out, closure
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

(* TESTS *)

(* let%test_module _ = 
   (module struct
    open Workload

    let multi_datapack = Datapack.create 2
    let single_datapack = Datapack.create 1


    let%test _ = (
      try (validate (input single_datapack); false)
      with MustHaveAtLeastOneJob -> true
    )



   end) *)

