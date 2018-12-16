[@@@ocaml.warning "-27-30-39"]


type data_retrieval_request = {
  user_id : string;
  job_id : int32;
}

type data_retrieval_response = {
  bytes : bytes;
}

let rec default_data_retrieval_request 
  ?user_id:((user_id:string) = "")
  ?job_id:((job_id:int32) = 0l)
  () : data_retrieval_request  = {
  user_id;
  job_id;
}

let rec default_data_retrieval_response 
  ?bytes:((bytes:bytes) = Bytes.create 0)
  () : data_retrieval_response  = {
  bytes;
}
