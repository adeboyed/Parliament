[@@@ocaml.warning "-27-30-39"]


type user_job_status_request = {
  user_id : string;
  job_ids : int32 list;
}

type user_job_status_status =
  | Blocked 
  | Queued 
  | Running 
  | Completed 
  | Halted 
  | Cancelled 

type user_job_status = {
  job_id : int32;
  status : user_job_status_status;
}

type user_job_status_response = {
  job_statuses : user_job_status list;
}

let rec default_user_job_status_request 
  ?user_id:((user_id:string) = "")
  ?job_ids:((job_ids:int32 list) = [])
  () : user_job_status_request  = {
  user_id;
  job_ids;
}

let rec default_user_job_status_status () = (Blocked:user_job_status_status)

let rec default_user_job_status 
  ?job_id:((job_id:int32) = 0l)
  ?status:((status:user_job_status_status) = default_user_job_status_status ())
  () : user_job_status  = {
  job_id;
  status;
}

let rec default_user_job_status_response 
  ?job_statuses:((job_statuses:user_job_status list) = [])
  () : user_job_status_response  = {
  job_statuses;
}
