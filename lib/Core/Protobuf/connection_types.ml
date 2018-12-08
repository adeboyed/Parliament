[@@@ocaml.warning "-27-30-39"]


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

type single_response =
  | Job_status_response of Status_types.job_status_reponse
  | Data_retrieval_response of Data_types.data_retrieval_response

let rec default_connection_request_status () = (Heartbeat:connection_request_status)

let rec default_connection_request 
  ?user_id:((user_id:int32) = 0l)
  ?status:((status:connection_request_status) = default_connection_request_status ())
  () : connection_request  = {
  user_id;
  status;
}

let rec default_single_request () : single_request = Connection_request (default_connection_request ())

let rec default_single_response () : single_response = Job_status_response (Status_types.default_job_status_reponse ())
