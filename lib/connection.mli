(** Function that handles sending a Protobuf single request to a given hostname and port and returning a response *)

exception ConnectionError of string
(** An exception thrown if an error is encountered at any part of the request/response decoding/encoding + transmissions *)

val send_single_request : string -> int -> Parliament_proto.Connection_types.single_user_request -> Parliament_proto.Connection_types.single_user_response
(** [send_single_request hostname port request] sends a Protobuf Single Request object using TCP sockets to the server given by the hostname and the port number *)

val send_worker_request : string -> int -> Parliament_proto.Worker_types.single_worker_request -> Parliament_proto.Worker_types.single_worker_response
(** [send_worker_request hostname port request] sends a Protobuf Single Request object using TCP sockets to the server given by the hostname and the port number *)