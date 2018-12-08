(** Create_connection.proto Binary Encoding *)


(** {2 Protobuf Encoding} *)

val encode_create_connection_request : Create_connection_types.create_connection_request -> Pbrt.Encoder.t -> unit
(** [encode_create_connection_request v encoder] encodes [v] with the given [encoder] *)

val encode_create_connection_response : Create_connection_types.create_connection_response -> Pbrt.Encoder.t -> unit
(** [encode_create_connection_response v encoder] encodes [v] with the given [encoder] *)


(** {2 Protobuf Decoding} *)

val decode_create_connection_request : Pbrt.Decoder.t -> Create_connection_types.create_connection_request
(** [decode_create_connection_request decoder] decodes a [create_connection_request] value from [decoder] *)

val decode_create_connection_response : Pbrt.Decoder.t -> Create_connection_types.create_connection_response
(** [decode_create_connection_response decoder] decodes a [create_connection_response] value from [decoder] *)
