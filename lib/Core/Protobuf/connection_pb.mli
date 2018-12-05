(** connection.proto Binary Encoding *)


(** {2 Protobuf Encoding} *)

val encode_connection_request_status : Connection_types.connection_request_status -> Pbrt.Encoder.t -> unit
(** [encode_connection_request_status v encoder] encodes [v] with the given [encoder] *)

val encode_connection_request : Connection_types.connection_request -> Pbrt.Encoder.t -> unit
(** [encode_connection_request v encoder] encodes [v] with the given [encoder] *)

val encode_single_request : Connection_types.single_request -> Pbrt.Encoder.t -> unit
(** [encode_single_request v encoder] encodes [v] with the given [encoder] *)


(** {2 Protobuf Decoding} *)

val decode_connection_request_status : Pbrt.Decoder.t -> Connection_types.connection_request_status
(** [decode_connection_request_status decoder] decodes a [connection_request_status] value from [decoder] *)

val decode_connection_request : Pbrt.Decoder.t -> Connection_types.connection_request
(** [decode_connection_request decoder] decodes a [connection_request] value from [decoder] *)

val decode_single_request : Pbrt.Decoder.t -> Connection_types.single_request
(** [decode_single_request decoder] decodes a [single_request] value from [decoder] *)
