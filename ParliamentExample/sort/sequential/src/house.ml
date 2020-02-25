open Printf

exception IO_ERROR

let load_file f =
  let ic = open_in f in
  let n = in_channel_length ic in
  let input = Bytes.create n in
  really_input ic input 0 n;
  close_in ic;
  Core.String.split_on_chars ~on:['\t' ; '\n' ; '\r' ] input

let analyse input_file output_file =
  Printf.printf "Running MapReduce for file %s \n%!" input_file;
  let input = load_file input_file in
  let sorted_list = List.sort compare input in 
  let oc = open_out output_file in
  List.iter (fun x -> fprintf oc "%s\n" x) sorted_list; 
  close_out oc

let () =
  let inputargs = Array.get Sys.argv 1 in
  let input_dir, output_dir = match (Core.String.split ~on:':' inputargs) with
     [x ; y] -> (x, y)
    | _ -> raise IO_ERROR
  in
  let files = Sys.readdir input_dir in
  Array.iter (fun x -> analyse (input_dir ^ "/" ^ x) (output_dir ^ "/" ^ x) ) files
