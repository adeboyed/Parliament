(** Worker.proto Binary Encoding *)


(** {2 Protobuf Encoding} *)

val encode_worker_input_map_type : Worker_types.worker_input_map_type -> Pbrt.Encoder.t -> unit
(** [encode_worker_input_map_type v encoder] encodes [v] with the given [encoder] *)

val encode_worker_input : Worker_types.worker_input -> Pbrt.Encoder.t -> unit
(** [encode_worker_input v encoder] encodes [v] with the given [encoder] *)

val encode_worker_output : Worker_types.worker_output -> Pbrt.Encoder.t -> unit
(** [encode_worker_output v encoder] encodes [v] with the given [encoder] *)


(** {2 Protobuf Decoding} *)

val decode_worker_input_map_type : Pbrt.Decoder.t -> Worker_types.worker_input_map_type
(** [decode_worker_input_map_type decoder] decodes a [worker_input_map_type] value from [decoder] *)

val decode_worker_input : Pbrt.Decoder.t -> Worker_types.worker_input
(** [decode_worker_input decoder] decodes a [worker_input] value from [decoder] *)

val decode_worker_output : Pbrt.Decoder.t -> Worker_types.worker_output
(** [decode_worker_output decoder] decodes a [worker_output] value from [decoder] *)
