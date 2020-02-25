open Printf

open Parliament
open Parliament.Door
open Parliament.Door.Workload
open Parliament.Door.Context

(* SPLIT PHASE *)

exception IO_ERROR
exception JobDidNotSubmit

(* MAP PHASE *)
let split_function dp = 
  let input_file : string = Datapack.get_item dp 0 in
  Printf.printf "Starting Split for file %s \n%!" input_file;
  let ic = open_in input_file in
  let n = in_channel_length ic in
  let input = Bytes.create n in
  really_input ic input 0 n;
  close_in ic;
  let num_strings = Core.String.split_on_chars ~on:['\t' ; '\n' ; '\r' ] input in
  let word_tbl = Hashtbl.create 26 in

  (* Insert things *)
  let chars = List.map Char.chr (Core.List.init 26 ~f:(fun x -> x + 97)) in
    List.iter (fun x -> Hashtbl.add word_tbl x ([]:string list)) chars;
  
  let insert_to_hash x =
    if String.length x > 0 then
      let first_char = String.get x 0 in 
      Hashtbl.replace word_tbl first_char (x::(Hashtbl.find word_tbl first_char)) 
  in

  let char_to_int c = Char.code c - 97 in
  
  List.iter insert_to_hash num_strings;
  let datapack_out = Datapack.create 26 in 

  Hashtbl.iter (fun ch lst -> Datapack.add_item (datapack_out) (lst) (char_to_int ch)) word_tbl;
  datapack_out

let map_function input_file dp = 
  let lst = Datapack.get_item dp 0 in 
  Printf.printf "Starting Map for file %s \n!" input_file;
  let sorted_list = List.sort compare lst in
  Datapack.single_item sorted_list

let reduce_function output_file dp = 
  Printf.printf "Starting Reduce for file %s \n%!" output_file;
  let length = Datapack.length dp in
  let ids = Core.List.init length ~f:(fun x -> x) in
  let oc = open_out output_file in
  let write_to_file x =
    let lst = Datapack.get_item dp x in
    List.iter (fun x -> fprintf oc "%s\n" x) lst in
  List.iter (fun x -> write_to_file x) ids;
  close_out oc;
  Datapack.single_item 1

let dist_sort ctx name input_dir output_dir =
  let input_file = input_dir ^ "/" ^ name in 
  let output_file =  output_dir ^ "/" ^ name in 
  Printf.printf "Starting MapReduce for file %s\n%!" input_file;
  let input_dp = Datapack.single_item input_file in
  let jobs = [SingleInMultiOut(split_function); SingleInSingleOut(map_function input_file); MultiInSingleOut(reduce_function output_file)] in
  let workload = Workload.add_all (Workload.input input_dp) jobs in
  match submit ctx workload with
    | Some(jobs) -> jobs
    | None -> raise JobDidNotSubmit

let () =
  let (ctx, a) = House.init() in
  let input_dir, output_dir = match (Core.String.split ~on:':' a) with
     [x ; y] -> (x, y)
    | _ -> raise JobDidNotSubmit
  in
  let files = Sys.readdir input_dir in
  let jobs_array = Array.map (fun x -> dist_sort ctx x input_dir output_dir) files in
  let jobs_list = List.flatten (Array.to_list jobs_array) in 
  wait_until_output ctx jobs_list
