(** data.proto Binary Encoding *)


(** {2 Protobuf Encoding} *)

val encode_data_retrieval_request : Data_types.data_retrieval_request -> Pbrt.Encoder.t -> unit
(** [encode_data_retrieval_request v encoder] encodes [v] with the given [encoder] *)

val encode_data_retrieval_response : Data_types.data_retrieval_response -> Pbrt.Encoder.t -> unit
(** [encode_data_retrieval_response v encoder] encodes [v] with the given [encoder] *)


(** {2 Protobuf Decoding} *)

val decode_data_retrieval_request : Pbrt.Decoder.t -> Data_types.data_retrieval_request
(** [decode_data_retrieval_request decoder] decodes a [data_retrieval_request] value from [decoder] *)

val decode_data_retrieval_response : Pbrt.Decoder.t -> Data_types.data_retrieval_response
(** [decode_data_retrieval_response decoder] decodes a [data_retrieval_response] value from [decoder] *)
