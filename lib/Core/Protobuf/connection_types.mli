(** connection.proto Types *)



(** {2 Types} *)

type connection_request_status =
  | Heartbeat 
  | Close_connection 

type connection_request = {
  user_id : int32;
  status : connection_request_status;
}

type single_request =
  | Connection_request of connection_request
  | Job_submission of Job_types.job_submission
  | Data_retrieval_request of Data_types.data_retrieval_request
  | Job_status_request of Status_types.job_status_request


(** {2 Default values} *)

val default_connection_request_status : unit -> connection_request_status
(** [default_connection_request_status ()] is the default value for type [connection_request_status] *)

val default_connection_request : 
  ?user_id:int32 ->
  ?status:connection_request_status ->
  unit ->
  connection_request
(** [default_connection_request ()] is the default value for type [connection_request] *)

val default_single_request : unit -> single_request
(** [default_single_request ()] is the default value for type [single_request] *)
