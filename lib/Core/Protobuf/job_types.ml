[@@@ocaml.warning "-27-30-39"]


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

let rec default_input_action 
  ?data_loc_in:((data_loc_in:bytes) = Bytes.create 0)
  () : input_action  = {
  data_loc_in;
}

let rec default_map_action 
  ?data_loc_in:((data_loc_in:int32 list) = [])
  ?function_name:((function_name:string) = "")
  ?data_loc_out:((data_loc_out:int32) = 0l)
  () : map_action  = {
  data_loc_in;
  function_name;
  data_loc_out;
}

let rec default_output_action 
  ?data_out:((data_out:int32) = 0l)
  () : output_action  = {
  data_out;
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
  ?function_name:((function_name:string) = "")
  ?jobs:((jobs:job list) = [])
  () : job_submission  = {
  user_id;
  function_name;
  jobs;
}

let rec default_job_submission_response 
  ?job_accepted:((job_accepted:bool) = false)
  () : job_submission_response  = {
  job_accepted;
}
