(** create_connection.proto Types *)



(** {2 Types} *)

type create_connection_request = {
  authentication : string;
}

type create_connection_response = {
  user_id : string;
  connection_accepted : bool;
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
