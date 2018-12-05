(** job.proto Types *)



(** {2 Types} *)

type input_action = {
  data_loc_in : bytes;
}

type map_action = {
  data_loc_in : int32 list;
  function_name : string;
  data_loc_out : int32;
}

type output_action = {
  data_out : int32;
}

type job_action =
  | Input of input_action
  | Map of map_action
  | Output of output_action

and job = {
  job_id : int32;
  action : job_action;
}

type job_submission = {
  user_id : string;
  function_name : string;
  jobs : job list;
}

type job_submission_response = {
  job_accepted : bool;
}


(** {2 Default values} *)

val default_input_action : 
  ?data_loc_in:bytes ->
  unit ->
  input_action
(** [default_input_action ()] is the default value for type [input_action] *)

val default_map_action : 
  ?data_loc_in:int32 list ->
  ?function_name:string ->
  ?data_loc_out:int32 ->
  unit ->
  map_action
(** [default_map_action ()] is the default value for type [map_action] *)

val default_output_action : 
  ?data_out:int32 ->
  unit ->
  output_action
(** [default_output_action ()] is the default value for type [output_action] *)

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
  ?function_name:string ->
  ?jobs:job list ->
  unit ->
  job_submission
(** [default_job_submission ()] is the default value for type [job_submission] *)

val default_job_submission_response : 
  ?job_accepted:bool ->
  unit ->
  job_submission_response
(** [default_job_submission_response ()] is the default value for type [job_submission_response] *)
