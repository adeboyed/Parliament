open Printf

exception IO_ERROR

(* SPLIT PHASE *)
let load_file f =
  let ic = open_in f in
  let n = in_channel_length ic in
  let input = Bytes.create n in
  really_input ic input 0 n;
  close_in ic;
  let lines = Core.String.split_on_chars ~on:['\t' ; '\n' ; '\r' ] input in
  List.filter (fun x -> (String.length (String.trim x)) > 0) lines

(* MAP/REDUCE PHASE *)
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

let map_reduce lines =
  let tbl = Hashtbl.create 200 in
  let drop_non_alpha s = String.length s == String.length (keep_alpha s) in
  let map_function input = 
    let words_in  = (Core.String.split_on_chars ~on:[ ' '] (String.lowercase_ascii input))in 
    let words_inter = List.filter drop_non_alpha words_in in
    let words_out = List.filter (fun x -> (String.length x) > 0) words_inter in
    create_tbl tbl words_out
   in
  List.iter (fun x -> map_function x) lines;
  tbl

let analyse input_file output_file =
  Printf.printf "Running MapReduce for file %s \n%!" input_file;
  let split_output = load_file input_file in
  let map_output = map_reduce split_output in
  let oc = open_out output_file in
  Hashtbl.iter (fun (x:string) (y:int) -> fprintf oc "%s -> %s\n" x (string_of_int y)) map_output;
  close_out oc

let () =
  let inputargs = Array.get Sys.argv 1 in
  let input_dir, output_dir = match (Core.String.split ~on:':' inputargs) with
     [x ; y] -> (x, y)
    | _ -> raise IO_ERROR
  in
  let files = Sys.readdir input_dir in
  Array.iter (fun x -> analyse (input_dir ^ "/" ^ x) (output_dir ^ "/" ^ x) ) files
