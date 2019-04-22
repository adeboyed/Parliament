(** Job.proto Types *)



(** {2 Types} *)

type input_action = {
  data_loc_in : bytes list;
}

type map_action_map_type =
  | Single_in_multi_out 
  | Single_in_single_out 
  | Multi_in_single_out 

type map_action = {
  map_type : map_action_map_type;
  job_id_in : int32;
  function_closure : bytes;
}

type job_action =
  | Input of input_action
  | Map of map_action

and job = {
  job_id : int32;
  action : job_action;
}

type job_submission = {
  user_id : string;
  jobs : job list;
}

type job_submission_response = {
  job_accepted : bool;
}


(** {2 Default values} *)

val default_input_action : 
  ?data_loc_in:bytes list ->
  unit ->
  input_action
(** [default_input_action ()] is the default value for type [input_action] *)

val default_map_action_map_type : unit -> map_action_map_type
(** [default_map_action_map_type ()] is the default value for type [map_action_map_type] *)

val default_map_action : 
  ?map_type:map_action_map_type ->
  ?job_id_in:int32 ->
  ?function_closure:bytes ->
  unit ->
  map_action
(** [default_map_action ()] is the default value for type [map_action] *)

val default_job_action : unit -> job_action
(** [default_job_action ()] is the default value for type [job_action] *)

val default_job : 
  ?job_id:int32 ->
  ?action:job_action ->
  unit ->
  job
(** [default_job ()] is the default value for type [job] *)

val default_job_submission : 
  ?user_id:string ->
  ?jobs:job list ->
  unit ->
  job_submission
(** [default_job_submission ()] is the default value for type [job_submission] *)

val default_job_submission_response : 
  ?job_accepted:bool ->
  unit ->
  job_submission_response
(** [default_job_submission_response ()] is the default value for type [job_submission_response] *)
