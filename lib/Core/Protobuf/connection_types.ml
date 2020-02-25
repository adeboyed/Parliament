[@@@ocaml.warning "-27-30-39"]


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

let rec default_connection_request_action () = (Heartbeat:connection_request_action)

let rec default_connection_request 
  ?user_id:((user_id:string) = "")
  ?action:((action:connection_request_action) = default_connection_request_action ())
  () : connection_request  = {
  user_id;
  action;
}

let rec default_connection_response 
  ?request_accepted:((request_accepted:bool) = false)
  () : connection_response  = {
  request_accepted;
}

let rec default_server_message_action () = (User_timeout:server_message_action)

let rec default_server_message 
  ?action:((action:server_message_action) = default_server_message_action ())
  () : server_message  = {
  action;
}

let rec default_single_user_request () : single_user_request = Create_connection_request (Create_connection_types.default_create_connection_request ())

let rec default_single_user_response () : single_user_response = Create_connection_response (Create_connection_types.default_create_connection_response ())
