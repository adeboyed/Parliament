(** Status.proto Types *)



(** {2 Types} *)

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


(** {2 Default values} *)

val default_user_job_status_request : 
  ?user_id:string ->
  ?job_ids:int32 list ->
  unit ->
  user_job_status_request
(** [default_user_job_status_request ()] is the default value for type [user_job_status_request] *)

val default_user_job_status_status : unit -> user_job_status_status
(** [default_user_job_status_status ()] is the default value for type [user_job_status_status] *)

val default_user_job_status : 
  ?job_id:int32 ->
  ?status:user_job_status_status ->
  unit ->
  user_job_status
(** [default_user_job_status ()] is the default value for type [user_job_status] *)

val default_user_job_status_response : 
  ?job_statuses:user_job_status list ->
  unit ->
  user_job_status_response
(** [default_user_job_status_response ()] is the default value for type [user_job_status_response] *)
