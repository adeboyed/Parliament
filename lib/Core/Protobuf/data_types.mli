(** Data.proto Types *)



(** {2 Types} *)

type data_retrieval_request = {
  job_id : int32;
}

type data_retrieval_response = {
  bytes : bytes list;
}


(** {2 Default values} *)

val default_data_retrieval_request : 
  ?job_id:int32 ->
  unit ->
  data_retrieval_request
(** [default_data_retrieval_request ()] is the default value for type [data_retrieval_request] *)

val default_data_retrieval_response : 
  ?bytes:bytes list ->
  unit ->
  data_retrieval_response
(** [default_data_retrieval_response ()] is the default value for type [data_retrieval_response] *)
