(** Job.proto Binary Encoding *)


(** {2 Protobuf Encoding} *)

val encode_input_action : Job_types.input_action -> Pbrt.Encoder.t -> unit
(** [encode_input_action v encoder] encodes [v] with the given [encoder] *)

val encode_map_action_map_type : Job_types.map_action_map_type -> Pbrt.Encoder.t -> unit
(** [encode_map_action_map_type v encoder] encodes [v] with the given [encoder] *)

val encode_map_action : Job_types.map_action -> Pbrt.Encoder.t -> unit
(** [encode_map_action v encoder] encodes [v] with the given [encoder] *)

val encode_job_action : Job_types.job_action -> Pbrt.Encoder.t -> unit
(** [encode_job_action v encoder] encodes [v] with the given [encoder] *)

val encode_job : Job_types.job -> Pbrt.Encoder.t -> unit
(** [encode_job v encoder] encodes [v] with the given [encoder] *)

val encode_job_submission : Job_types.job_submission -> Pbrt.Encoder.t -> unit
(** [encode_job_submission v encoder] encodes [v] with the given [encoder] *)

val encode_job_submission_response : Job_types.job_submission_response -> Pbrt.Encoder.t -> unit
(** [encode_job_submission_response v encoder] encodes [v] with the given [encoder] *)


(** {2 Protobuf Decoding} *)

val decode_input_action : Pbrt.Decoder.t -> Job_types.input_action
(** [decode_input_action decoder] decodes a [input_action] value from [decoder] *)

val decode_map_action_map_type : Pbrt.Decoder.t -> Job_types.map_action_map_type
(** [decode_map_action_map_type decoder] decodes a [map_action_map_type] value from [decoder] *)

val decode_map_action : Pbrt.Decoder.t -> Job_types.map_action
(** [decode_map_action decoder] decodes a [map_action] value from [decoder] *)

val decode_job_action : Pbrt.Decoder.t -> Job_types.job_action
(** [decode_job_action decoder] decodes a [job_action] value from [decoder] *)

val decode_job : Pbrt.Decoder.t -> Job_types.job
(** [decode_job decoder] decodes a [job] value from [decoder] *)

val decode_job_submission : Pbrt.Decoder.t -> Job_types.job_submission
(** [decode_job_submission decoder] decodes a [job_submission] value from [decoder] *)

val decode_job_submission_response : Pbrt.Decoder.t -> Job_types.job_submission_response
(** [decode_job_submission_response decoder] decodes a [job_submission_response] value from [decoder] *)
