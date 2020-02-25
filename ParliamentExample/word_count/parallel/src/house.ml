open Printf

open Parliament
open Parliament.Door
open Parliament.Door.Workload
open Parliament.Door.Context

(* SPLIT PHASE *)

exception IO_ERROR
exception JobDidNotSubmit

let segments xs n = 
  let rec loop n xs (count, elem) acc = 
      match xs with
      | x::xs' when count < n -> loop n xs' ((count + String.length x) , x::elem) acc
      | x::xs' -> loop n xs' (String.length x, [x]) ((List.rev elem)::acc)
      | [] -> List.rev ((List.rev elem)::acc) in
  loop n xs (0, []) []

let split_function block_size dp =
  let input : string = Datapack.get_item dp 0 in
  let ic = open_in input in
  let n = in_channel_length ic in
  let input = Bytes.create n in
  really_input ic input 0 n;
  close_in ic;
  let lines = Core.String.split_on_chars ~on:['\t' ; '\n' ; '\r' ] input in
  let output = List.filter (fun x -> (String.length (String.trim x)) > 0) lines in 
  let segs = segments output block_size in 
  Datapack.from_list segs

(* MAP PHASE *)
let create_tbl tbl words = 
  let add_word word =
    try (
         let current_no = Hashtbl.find tbl word in 
         Hashtbl.replace tbl word (current_no+1)
       ) with Not_found -> Hashtbl.add tbl word 1
  in
  List.iter (fun x -> add_word x) words

let keep_alpha s =
  let b = Buffer.create 255 in
  let add_alpha () _ = function
  | `Malformed _ -> Uutf.Buffer.add_utf_8 b Uutf.u_rep
  | `Uchar u -> if Uucp.Alpha.is_alphabetic u then Uutf.Buffer.add_utf_8 b u
  in
  Uutf.String.fold_utf_8 add_alpha () s;
  Buffer.contents b

let map_function file dp = 
  Printf.printf "Starting Map for file %s \n%!" file;
  let tbl = Hashtbl.create 200 in
  let lines : string list = Datapack.get_item dp 0 in
  let drop_non_alpha s = String.length s == String.length (keep_alpha s) in
  let map input = 
    let words_in  = (Core.String.split_on_chars ~on:[ ' '] (String.lowercase_ascii input)) in 
    let words_inter = List.filter drop_non_alpha words_in in
    let words_out = List.filter (fun x -> (String.length x) > 0) words_inter in
     create_tbl tbl words_out;
  in
  List.iter (fun x -> map x) lines;
  Datapack.single_item tbl

(* REDUCE PHASE *)

let merge tbl1 tbl2 =
  let merge_element (word:string) (count:int) = 
    try (
      let current_no = Hashtbl.find tbl1 word in
      Hashtbl.replace tbl1 word (current_no+count)
    ) with Not_found -> (Hashtbl.add tbl1 word count)
  in
  Hashtbl.iter merge_element tbl2

let reduce_function output_file dp = 
  Printf.printf "Starting Reduce for file %s \n%!" output_file;
  let length = Datapack.length dp in
  let tbl = Datapack.get_item dp 0 in
  (for i = 0 to (length-1) do (merge tbl (Datapack.get_item dp i)) done);
  let oc = open_out output_file in
  Hashtbl.iter (fun (x:string) (y:int) -> fprintf oc "%s -> %s\n" x (string_of_int y)) tbl;
  close_out oc;
  Datapack.create 1

let analyse ctx name input_dir output_dir block_size =
  let input_file = input_dir ^ "/" ^ name in 
  let output_file =  output_dir ^ "/" ^ name in 
  Printf.printf "Starting MapReduce for file %s Block Size: %d \n%!" input_file block_size;
  let input_dp = Datapack.single_item input_file in
  let jobs = [SingleInMultiOut(split_function block_size); SingleInSingleOut(map_function name); MultiInSingleOut(reduce_function output_file)] in
  let workload = Workload.add_all (Workload.input input_dp) jobs in
  match submit ctx workload with
    | Some(jobs) -> jobs
    | None -> raise JobDidNotSubmit

let () =
  let (ctx, a) = House.init() in
  let input_dir, output_dir, block_size = match (Core.String.split ~on:':' a) with
     [x ; y ; z ] -> (x, y, z)
    | _ -> raise JobDidNotSubmit
  in
  let files = Sys.readdir input_dir in
  let jobs_array = Array.map (fun x -> analyse ctx x input_dir output_dir (int_of_string block_size)) files in
  let jobs_list = List.flatten (Array.to_list jobs_array) in 
  wait_until_output ctx jobs_list

(* Size = 10000000 *) 