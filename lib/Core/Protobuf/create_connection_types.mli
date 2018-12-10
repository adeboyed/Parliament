(** Create_connection.proto Types *)



(** {2 Types} *)

type create_connection_request = {
  authentication : string;
}

type create_connection_response = {
  user_id : string;
  connection_accepted : bool;
}

type executable_request = {
  user_id : string;
  executable : bytes;
}


(** {2 Default values} *)

val default_create_connection_request : 
  ?authentication:string ->
  unit ->
  create_connection_request
(** [default_create_connection_request ()] is the default value for type [create_connection_request] *)

val default_create_connection_response : 
  ?user_id:string ->
  ?connection_accepted:bool ->
  unit ->
  create_connection_response
(** [default_create_connection_response ()] is the default value for type [create_connection_response] *)

val default_executable_request : 
  ?user_id:string ->
  ?executable:bytes ->
  unit ->
  executable_request
(** [default_executable_request ()] is the default value for type [executable_request] *)
