(** Status.proto Binary Encoding *)


(** {2 Protobuf Encoding} *)

val encode_user_job_status_request : Status_types.user_job_status_request -> Pbrt.Encoder.t -> unit
(** [encode_user_job_status_request v encoder] encodes [v] with the given [encoder] *)

val encode_user_job_status_status : Status_types.user_job_status_status -> Pbrt.Encoder.t -> unit
(** [encode_user_job_status_status v encoder] encodes [v] with the given [encoder] *)

val encode_user_job_status : Status_types.user_job_status -> Pbrt.Encoder.t -> unit
(** [encode_user_job_status v encoder] encodes [v] with the given [encoder] *)

val encode_user_job_status_response : Status_types.user_job_status_response -> Pbrt.Encoder.t -> unit
(** [encode_user_job_status_response v encoder] encodes [v] with the given [encoder] *)


(** {2 Protobuf Decoding} *)

val decode_user_job_status_request : Pbrt.Decoder.t -> Status_types.user_job_status_request
(** [decode_user_job_status_request decoder] decodes a [user_job_status_request] value from [decoder] *)

val decode_user_job_status_status : Pbrt.Decoder.t -> Status_types.user_job_status_status
(** [decode_user_job_status_status decoder] decodes a [user_job_status_status] value from [decoder] *)

val decode_user_job_status : Pbrt.Decoder.t -> Status_types.user_job_status
(** [decode_user_job_status decoder] decodes a [user_job_status] value from [decoder] *)

val decode_user_job_status_response : Pbrt.Decoder.t -> Status_types.user_job_status_response
(** [decode_user_job_status_response decoder] decodes a [user_job_status_response] value from [decoder] *)
