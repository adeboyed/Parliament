[@@@ocaml.warning "-27-30-39"]


type create_connection_request = {
  authentication : string;
  docker_name : string;
}

type create_connection_response = {
  user_id : string;
  connection_accepted : bool;
}

type executable_request = {
  user_id : string;
  executable : bytes;
}

let rec default_create_connection_request 
  ?authentication:((authentication:string) = "")
  ?docker_name:((docker_name:string) = "")
  () : create_connection_request  = {
  authentication;
  docker_name;
}

let rec default_create_connection_response 
  ?user_id:((user_id:string) = "")
  ?connection_accepted:((connection_accepted:bool) = false)
  () : create_connection_response  = {
  user_id;
  connection_accepted;
}

let rec default_executable_request 
  ?user_id:((user_id:string) = "")
  ?executable:((executable:bytes) = Bytes.create 0)
  () : executable_request  = {
  user_id;
  executable;
}
