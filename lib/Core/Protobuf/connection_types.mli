(** Connection.proto Types *)



(** {2 Types} *)

type connection_request_action =
  | Heartbeat 
  | Close_connection 

type connection_request = {
  user_id : string;
  action : connection_request_action;
}

type connection_response = {
  request_accepted : bool;
}

type server_message_action =
  | User_timeout 
  | Missing_jobs 
  | Internal_server_error 

type server_message = {
  action : server_message_action;
}

type single_user_request =
  | Create_connection_request of Create_connection_types.create_connection_request
  | Connection_request of connection_request
  | Job_submission of Job_types.job_submission
  | Data_retrieval_request of Data_types.data_retrieval_request
  | Job_status_request of Status_types.job_status_request
  | Executable_request of Create_connection_types.executable_request

type single_user_response =
  | Create_connection_response of Create_connection_types.create_connection_response
  | Job_submission_response of Job_types.job_submission_response
  | Data_retrieval_response of Data_types.data_retrieval_response
  | Job_status_response of Status_types.job_status_response
  | Connection_response of connection_response
  | Server_message of server_message


(** {2 Default values} *)

val default_connection_request_action : unit -> connection_request_action
(** [default_connection_request_action ()] is the default value for type [connection_request_action] *)

val default_connection_request : 
  ?user_id:string ->
  ?action:connection_request_action ->
  unit ->
  connection_request
(** [default_connection_request ()] is the default value for type [connection_request] *)

val default_connection_response : 
  ?request_accepted:bool ->
  unit ->
  connection_response
(** [default_connection_response ()] is the default value for type [connection_response] *)

val default_server_message_action : unit -> server_message_action
(** [default_server_message_action ()] is the default value for type [server_message_action] *)

val default_server_message : 
  ?action:server_message_action ->
  unit ->
  server_message
(** [default_server_message ()] is the default value for type [server_message] *)

val default_single_user_request : unit -> single_user_request
(** [default_single_user_request ()] is the default value for type [single_user_request] *)

val default_single_user_response : unit -> single_user_response
(** [default_single_user_response ()] is the default value for type [single_user_response] *)
