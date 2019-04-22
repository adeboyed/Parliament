[@@@ocaml.warning "-27-30-39"]

type job_status_request_mutable = {
  mutable user_id : string;
  mutable job_ids : int32 list;
}

let default_job_status_request_mutable () : job_status_request_mutable = {
  user_id = "";
  job_ids = [];
}

type job_status_mutable = {
  mutable job_id : int32;
  mutable status : Status_types.job_status_status;
}

let default_job_status_mutable () : job_status_mutable = {
  job_id = 0l;
  status = Status_types.default_job_status_status ();
}

type job_status_response_mutable = {
  mutable job_statuses : Status_types.job_status list;
}

let default_job_status_response_mutable () : job_status_response_mutable = {
  job_statuses = [];
}


let rec decode_job_status_request d =
  let v = default_job_status_request_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
      v.job_ids <- List.rev v.job_ids;
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.user_id <- Pbrt.Decoder.string d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_status_request), field(1)" pk
    | Some (2, Pbrt.Varint) -> begin
      v.job_ids <- (Pbrt.Decoder.int32_as_varint d) :: v.job_ids;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_status_request), field(2)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Status_types.user_id = v.user_id;
    Status_types.job_ids = v.job_ids;
  } : Status_types.job_status_request)

let rec decode_job_status_status d = 
  match Pbrt.Decoder.int_as_varint d with
  | 0 -> (Status_types.Blocked:Status_types.job_status_status)
  | 1 -> (Status_types.Queued:Status_types.job_status_status)
  | 2 -> (Status_types.Running:Status_types.job_status_status)
  | 4 -> (Status_types.Completed:Status_types.job_status_status)
  | 5 -> (Status_types.Halted:Status_types.job_status_status)
  | 6 -> (Status_types.Cancelled:Status_types.job_status_status)
  | _ -> Pbrt.Decoder.malformed_variant "job_status_status"

let rec decode_job_status d =
  let v = default_job_status_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (2, Pbrt.Varint) -> begin
      v.job_id <- Pbrt.Decoder.int32_as_varint d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_status), field(2)" pk
    | Some (3, Pbrt.Varint) -> begin
      v.status <- decode_job_status_status d;
    end
    | Some (3, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_status), field(3)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Status_types.job_id = v.job_id;
    Status_types.status = v.status;
  } : Status_types.job_status)

let rec decode_job_status_response d =
  let v = default_job_status_response_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
      v.job_statuses <- List.rev v.job_statuses;
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.job_statuses <- (decode_job_status (Pbrt.Decoder.nested d)) :: v.job_statuses;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_status_response), field(1)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Status_types.job_statuses = v.job_statuses;
  } : Status_types.job_status_response)

let rec encode_job_status_request (v:Status_types.job_status_request) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.string v.Status_types.user_id encoder;
  List.iter (fun x -> 
    Pbrt.Encoder.key (2, Pbrt.Varint) encoder; 
    Pbrt.Encoder.int32_as_varint x encoder;
  ) v.Status_types.job_ids;
  ()

let rec encode_job_status_status (v:Status_types.job_status_status) encoder =
  match v with
  | Status_types.Blocked -> Pbrt.Encoder.int_as_varint (0) encoder
  | Status_types.Queued -> Pbrt.Encoder.int_as_varint 1 encoder
  | Status_types.Running -> Pbrt.Encoder.int_as_varint 2 encoder
  | Status_types.Completed -> Pbrt.Encoder.int_as_varint 4 encoder
  | Status_types.Halted -> Pbrt.Encoder.int_as_varint 5 encoder
  | Status_types.Cancelled -> Pbrt.Encoder.int_as_varint 6 encoder

let rec encode_job_status (v:Status_types.job_status) encoder = 
  Pbrt.Encoder.key (2, Pbrt.Varint) encoder; 
  Pbrt.Encoder.int32_as_varint v.Status_types.job_id encoder;
  Pbrt.Encoder.key (3, Pbrt.Varint) encoder; 
  encode_job_status_status v.Status_types.status encoder;
  ()

let rec encode_job_status_response (v:Status_types.job_status_response) encoder = 
  List.iter (fun x -> 
    Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_job_status x) encoder;
  ) v.Status_types.job_statuses;
  ()
