[@@@ocaml.warning "-27-30-39"]


type job_status_request = {
  job_ids : int32 list;
}

type job_status_status =
  | Queued 
  | Waiting 
  | Running 
  | Completed 
  | Errored 
  | Cancelled 

type job_status = {
  job_id : int32;
  status : job_status_status;
}

type job_status_response = {
  job_status : job_status list;
}

let rec default_job_status_request 
  ?job_ids:((job_ids:int32 list) = [])
  () : job_status_request  = {
  job_ids;
}

let rec default_job_status_status () = (Queued:job_status_status)

let rec default_job_status 
  ?job_id:((job_id:int32) = 0l)
  ?status:((status:job_status_status) = default_job_status_status ())
  () : job_status  = {
  job_id;
  status;
}

let rec default_job_status_response 
  ?job_status:((job_status:job_status list) = [])
  () : job_status_response  = {
  job_status;
}
