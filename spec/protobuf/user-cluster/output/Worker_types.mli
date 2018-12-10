(** Worker.proto Types *)



(** {2 Types} *)

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


(** {2 Default values} *)

val default_worker_input_map_type : unit -> worker_input_map_type
(** [default_worker_input_map_type ()] is the default value for type [worker_input_map_type] *)

val default_worker_input : 
  ?function_closure:bytes ->
  ?map_type:worker_input_map_type ->
  ?datapack:bytes ->
  unit ->
  worker_input
(** [default_worker_input ()] is the default value for type [worker_input] *)

val default_worker_output : 
  ?datapack:bytes ->
  unit ->
  worker_output
(** [default_worker_output ()] is the default value for type [worker_output] *)
