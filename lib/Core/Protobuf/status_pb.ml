[@@@ocaml.warning "-27-30-39"]

type job_status_request_mutable = {
  mutable job_id : int32 list;
}

let default_job_status_request_mutable () : job_status_request_mutable = {
  job_id = [];
}

type job_status_mutable = {
  mutable job_id : int32;
  mutable status : Status_types.job_status_status;
}

let default_job_status_mutable () : job_status_mutable = {
  job_id = 0l;
  status = Status_types.default_job_status_status ();
}

type job_status_reponse_mutable = {
  mutable job_status : Status_types.job_status list;
}

let default_job_status_reponse_mutable () : job_status_reponse_mutable = {
  job_status = [];
}


let rec decode_job_status_request d =
  let v = default_job_status_request_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
      v.job_id <- List.rev v.job_id;
    ); continue__ := false
    | Some (1, Pbrt.Varint) -> begin
      v.job_id <- (Pbrt.Decoder.int32_as_varint d) :: v.job_id;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_status_request), field(1)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Status_types.job_id = v.job_id;
  } : Status_types.job_status_request)

let rec decode_job_status_status d = 
  match Pbrt.Decoder.int_as_varint d with
  | 0 -> (Status_types.Queued:Status_types.job_status_status)
  | 1 -> (Status_types.Waiting:Status_types.job_status_status)
  | 2 -> (Status_types.Running:Status_types.job_status_status)
  | 4 -> (Status_types.Completed:Status_types.job_status_status)
  | 5 -> (Status_types.Errored:Status_types.job_status_status)
  | _ -> Pbrt.Decoder.malformed_variant "job_status_status"

let rec decode_job_status d =
  let v = default_job_status_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Varint) -> begin
      v.job_id <- Pbrt.Decoder.int32_as_varint d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_status), field(1)" pk
    | Some (2, Pbrt.Varint) -> begin
      v.status <- decode_job_status_status d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_status), field(2)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Status_types.job_id = v.job_id;
    Status_types.status = v.status;
  } : Status_types.job_status)

let rec decode_job_status_reponse d =
  let v = default_job_status_reponse_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
      v.job_status <- List.rev v.job_status;
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.job_status <- (decode_job_status (Pbrt.Decoder.nested d)) :: v.job_status;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_status_reponse), field(1)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Status_types.job_status = v.job_status;
  } : Status_types.job_status_reponse)

let rec encode_job_status_request (v:Status_types.job_status_request) encoder = 
  List.iter (fun x -> 
    Pbrt.Encoder.key (1, Pbrt.Varint) encoder; 
    Pbrt.Encoder.int32_as_varint x encoder;
  ) v.Status_types.job_id;
  ()

let rec encode_job_status_status (v:Status_types.job_status_status) encoder =
  match v with
  | Status_types.Queued -> Pbrt.Encoder.int_as_varint (0) encoder
  | Status_types.Waiting -> Pbrt.Encoder.int_as_varint 1 encoder
  | Status_types.Running -> Pbrt.Encoder.int_as_varint 2 encoder
  | Status_types.Completed -> Pbrt.Encoder.int_as_varint 4 encoder
  | Status_types.Errored -> Pbrt.Encoder.int_as_varint 5 encoder

let rec encode_job_status (v:Status_types.job_status) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Varint) encoder; 
  Pbrt.Encoder.int32_as_varint v.Status_types.job_id encoder;
  Pbrt.Encoder.key (2, Pbrt.Varint) encoder; 
  encode_job_status_status v.Status_types.status encoder;
  ()

let rec encode_job_status_reponse (v:Status_types.job_status_reponse) encoder = 
  List.iter (fun x -> 
    Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_job_status x) encoder;
  ) v.Status_types.job_status;
  ()
