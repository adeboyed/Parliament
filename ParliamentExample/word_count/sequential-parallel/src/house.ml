open Printf

exception IO_ERROR

(* SPLIT PHASE *)
let segments xs n = 
  let rec loop n xs (count, elem) acc = 
      match xs with
      | x::xs' when count < n -> loop n xs' ((count + String.length x) , x::elem) acc
      | x::xs' -> loop n xs' (String.length x, [x]) ((List.rev elem)::acc)
      | [] -> List.rev ((List.rev elem)::acc) in
  loop n xs (0, []) []

let split_function block_size input =
  let ic = open_in input in
  let n = in_channel_length ic in
  let input = Bytes.create n in
  really_input ic input 0 n;
  close_in ic;
  let lines = Core.String.split_on_chars ~on:['\t' ; '\n' ; '\r' ] input in
  let output = List.filter (fun x -> (String.length (String.trim x)) > 0) lines in 
  let segs = segments output block_size in 
  segs

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

let map_function lines = 
  let tbl = Hashtbl.create 200 in
  let drop_non_alpha s = String.length s == String.length (keep_alpha s) in
  let map input = 
    let words_in  = (Core.String.split_on_chars ~on:[ ' '] (String.lowercase_ascii input)) in 
    let words_inter = List.filter drop_non_alpha words_in in
    let words_out = List.filter (fun x -> (String.length x) > 0) words_inter in
     create_tbl tbl words_out;
  in
  List.iter (fun x -> map x) lines;
  tbl

let merge tbl1 tbl2 =
  let merge_element (word:string) (count:int) = 
    try (
      let current_no = Hashtbl.find tbl1 word in
      Hashtbl.replace tbl1 word (current_no+count)
    ) with Not_found -> (Hashtbl.add tbl1 word count)
  in
  Hashtbl.iter merge_element tbl2

let reduce_function output_file = function
  (first::rest_tbl) ->
    List.iter (merge first) rest_tbl;
    let oc = open_out output_file in
    Hashtbl.iter (fun (x:string) (y:int) -> fprintf oc "%s -> %s\n" x (string_of_int y)) first;
    close_out oc
  | [] -> ()

let analyse block_size input_file output_file =
  Printf.printf "Running MapReduce for file %s \n%!" input_file;
  let split_output = split_function block_size input_file in
  let map_out = List.map map_function split_output in
  reduce_function output_file map_out

let () =
  let inputargs = Array.get Sys.argv 1 in
  let input_dir, output_dir, block_size = match (Core.String.split ~on:':' inputargs) with
     [x ; y ; z ] -> (x, y, z)
    | _ -> raise IO_ERROR
  in
  let files = Sys.readdir input_dir in
  Array.iter (fun x -> analyse (int_of_string block_size) (input_dir ^ "/" ^ x) (output_dir ^ "/" ^ x) ) files
