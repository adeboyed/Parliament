[@@@ocaml.warning "-27-30-39"]


type create_connection_request = {
  authentication : string;
}

type create_connection_response = {
  user_id : string;
  connection_accepted : bool;
}

let rec default_create_connection_request 
  ?authentication:((authentication:string) = "")
  () : create_connection_request  = {
  authentication;
}

let rec default_create_connection_response 
  ?user_id:((user_id:string) = "")
  ?connection_accepted:((connection_accepted:bool) = false)
  () : create_connection_response  = {
  user_id;
  connection_accepted;
}
