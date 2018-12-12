(** Connection.proto Binary Encoding *)


(** {2 Protobuf Encoding} *)

val encode_connection_request_action : Connection_types.connection_request_action -> Pbrt.Encoder.t -> unit
(** [encode_connection_request_action v encoder] encodes [v] with the given [encoder] *)

val encode_connection_request : Connection_types.connection_request -> Pbrt.Encoder.t -> unit
(** [encode_connection_request v encoder] encodes [v] with the given [encoder] *)

val encode_connection_response : Connection_types.connection_response -> Pbrt.Encoder.t -> unit
(** [encode_connection_response v encoder] encodes [v] with the given [encoder] *)

val encode_server_message_action : Connection_types.server_message_action -> Pbrt.Encoder.t -> unit
(** [encode_server_message_action v encoder] encodes [v] with the given [encoder] *)

val encode_server_message : Connection_types.server_message -> Pbrt.Encoder.t -> unit
(** [encode_server_message v encoder] encodes [v] with the given [encoder] *)

val encode_single_request : Connection_types.single_request -> Pbrt.Encoder.t -> unit
(** [encode_single_request v encoder] encodes [v] with the given [encoder] *)

val encode_single_response : Connection_types.single_response -> Pbrt.Encoder.t -> unit
(** [encode_single_response v encoder] encodes [v] with the given [encoder] *)


(** {2 Protobuf Decoding} *)

val decode_connection_request_action : Pbrt.Decoder.t -> Connection_types.connection_request_action
(** [decode_connection_request_action decoder] decodes a [connection_request_action] value from [decoder] *)

val decode_connection_request : Pbrt.Decoder.t -> Connection_types.connection_request
(** [decode_connection_request decoder] decodes a [connection_request] value from [decoder] *)

val decode_connection_response : Pbrt.Decoder.t -> Connection_types.connection_response
(** [decode_connection_response decoder] decodes a [connection_response] value from [decoder] *)

val decode_server_message_action : Pbrt.Decoder.t -> Connection_types.server_message_action
(** [decode_server_message_action decoder] decodes a [server_message_action] value from [decoder] *)

val decode_server_message : Pbrt.Decoder.t -> Connection_types.server_message
(** [decode_server_message decoder] decodes a [server_message] value from [decoder] *)

val decode_single_request : Pbrt.Decoder.t -> Connection_types.single_request
(** [decode_single_request decoder] decodes a [single_request] value from [decoder] *)

val decode_single_response : Pbrt.Decoder.t -> Connection_types.single_response
(** [decode_single_response decoder] decodes a [single_response] value from [decoder] *)
