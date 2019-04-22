(** Status.proto Types *)



(** {2 Types} *)

type job_status_request = {
  user_id : string;
  job_ids : int32 list;
}

type job_status_status =
  | Blocked 
  | Queued 
  | Running 
  | Completed 
  | Halted 
  | Cancelled 

type job_status = {
  job_id : int32;
  status : job_status_status;
}

type job_status_response = {
  job_statuses : job_status list;
}


(** {2 Default values} *)

val default_job_status_request : 
  ?user_id:string ->
  ?job_ids:int32 list ->
  unit ->
  job_status_request
(** [default_job_status_request ()] is the default value for type [job_status_request] *)

val default_job_status_status : unit -> job_status_status
(** [default_job_status_status ()] is the default value for type [job_status_status] *)

val default_job_status : 
  ?job_id:int32 ->
  ?status:job_status_status ->
  unit ->
  job_status
(** [default_job_status ()] is the default value for type [job_status] *)

val default_job_status_response : 
  ?job_statuses:job_status list ->
  unit ->
  job_status_response
(** [default_job_status_response ()] is the default value for type [job_status_response] *)
