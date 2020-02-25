open Parliament
open Parliament.Door
open Parliament.Door.Workload
open Parliament.Door.Context

open Core
open Unix

(* SPLIT PHASE *)

exception IO_ERROR
exception JobDidNotSubmit

let sleep_function dp =
  sleep 1;
  dp

let () =
  let (ctx, a) = House.init() in
  let job_count = int_of_string a in
  let job_input_list = List.init job_count ~f:(const 0) in
  let input_dp = Datapack.from_list job_input_list in
  let jobs = [SingleInSingleOut(sleep_function)] in
  let workload = Workload.add_all (Workload.input input_dp) jobs in
  let jobs_list = (match submit ctx workload with
  | Some(jobs) -> jobs
  | None -> raise JobDidNotSubmit) in
  wait_until_output ctx jobs_list

(* Size = 10000000 *) 