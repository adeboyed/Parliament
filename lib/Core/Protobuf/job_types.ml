[@@@ocaml.warning "-27-30-39"]


type input_action = {
  data_loc_in : bytes;
}

type map_action_map_type =
  | Single_in_variable_out 
  | Single_in_single_out 
  | Variable_in_variable_out 

type map_action = {
  map_type : map_action_map_type;
  job_id_in : int32;
  function_name : string;
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

let rec default_input_action 
  ?data_loc_in:((data_loc_in:bytes) = Bytes.create 0)
  () : input_action  = {
  data_loc_in;
}

let rec default_map_action_map_type () = (Single_in_variable_out:map_action_map_type)

let rec default_map_action 
  ?map_type:((map_type:map_action_map_type) = default_map_action_map_type ())
  ?job_id_in:((job_id_in:int32) = 0l)
  ?function_name:((function_name:string) = "")
  () : map_action  = {
  map_type;
  job_id_in;
  function_name;
}

let rec default_job_action () : job_action = Input (default_input_action ())

and default_job 
  ?job_id:((job_id:int32) = 0l)
  ?action:((action:job_action) = Input (default_input_action ()))
  () : job  = {
  job_id;
  action;
}

let rec default_job_submission 
  ?user_id:((user_id:string) = "")
  ?jobs:((jobs:job list) = [])
  () : job_submission  = {
  user_id;
  jobs;
}

let rec default_job_submission_response 
  ?job_accepted:((job_accepted:bool) = false)
  () : job_submission_response  = {
  job_accepted;
}
