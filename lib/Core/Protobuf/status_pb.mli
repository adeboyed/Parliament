(** Status.proto Binary Encoding *)


(** {2 Protobuf Encoding} *)

val encode_job_status_request : Status_types.job_status_request -> Pbrt.Encoder.t -> unit
(** [encode_job_status_request v encoder] encodes [v] with the given [encoder] *)

val encode_job_status_status : Status_types.job_status_status -> Pbrt.Encoder.t -> unit
(** [encode_job_status_status v encoder] encodes [v] with the given [encoder] *)

val encode_job_status : Status_types.job_status -> Pbrt.Encoder.t -> unit
(** [encode_job_status v encoder] encodes [v] with the given [encoder] *)

val encode_job_status_response : Status_types.job_status_response -> Pbrt.Encoder.t -> unit
(** [encode_job_status_response v encoder] encodes [v] with the given [encoder] *)


(** {2 Protobuf Decoding} *)

val decode_job_status_request : Pbrt.Decoder.t -> Status_types.job_status_request
(** [decode_job_status_request decoder] decodes a [job_status_request] value from [decoder] *)

val decode_job_status_status : Pbrt.Decoder.t -> Status_types.job_status_status
(** [decode_job_status_status decoder] decodes a [job_status_status] value from [decoder] *)

val decode_job_status : Pbrt.Decoder.t -> Status_types.job_status
(** [decode_job_status decoder] decodes a [job_status] value from [decoder] *)

val decode_job_status_response : Pbrt.Decoder.t -> Status_types.job_status_response
(** [decode_job_status_response decoder] decodes a [job_status_response] value from [decoder] *)
