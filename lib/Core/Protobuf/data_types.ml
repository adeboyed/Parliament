[@@@ocaml.warning "-27-30-39"]


type data_retrieval_request = {
  job_id : int32;
}

type data_retrieval_response = {
  bytes : bytes list;
}

let rec default_data_retrieval_request 
  ?job_id:((job_id:int32) = 0l)
  () : data_retrieval_request  = {
  job_id;
}

let rec default_data_retrieval_response 
  ?bytes:((bytes:bytes list) = [])
  () : data_retrieval_response  = {
  bytes;
}
