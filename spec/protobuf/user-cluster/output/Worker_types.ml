[@@@ocaml.warning "-27-30-39"]


type worker_input_map_type =
  | Single_in_variable_out 
  | Single_in_single_out 
  | Variable_in_variable_out 

type worker_input = {
  function_closure : bytes;
  map_type : worker_input_map_type;
  datapack : bytes;
}

type worker_output = {
  datapack : bytes;
}

let rec default_worker_input_map_type () = (Single_in_variable_out:worker_input_map_type)

let rec default_worker_input 
  ?function_closure:((function_closure:bytes) = Bytes.create 0)
  ?map_type:((map_type:worker_input_map_type) = default_worker_input_map_type ())
  ?datapack:((datapack:bytes) = Bytes.create 0)
  () : worker_input  = {
  function_closure;
  map_type;
  datapack;
}

let rec default_worker_output 
  ?datapack:((datapack:bytes) = Bytes.create 0)
  () : worker_output  = {
  datapack;
}
