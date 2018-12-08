[@@@ocaml.warning "-27-30-39"]

type connection_request_mutable = {
  mutable user_id : int32;
  mutable status : Connection_types.connection_request_status;
}

let default_connection_request_mutable () : connection_request_mutable = {
  user_id = 0l;
  status = Connection_types.default_connection_request_status ();
}


let rec decode_connection_request_status d = 
  match Pbrt.Decoder.int_as_varint d with
  | 0 -> (Connection_types.Heartbeat:Connection_types.connection_request_status)
  | 1 -> (Connection_types.Close_connection:Connection_types.connection_request_status)
  | _ -> Pbrt.Decoder.malformed_variant "connection_request_status"

let rec decode_connection_request d =
  let v = default_connection_request_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Varint) -> begin
      v.user_id <- Pbrt.Decoder.int32_as_varint d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(connection_request), field(1)" pk
    | Some (2, Pbrt.Varint) -> begin
      v.status <- decode_connection_request_status d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(connection_request), field(2)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Connection_types.user_id = v.user_id;
    Connection_types.status = v.status;
  } : Connection_types.connection_request)

let rec decode_single_request d = 
  let rec loop () = 
    let ret:Connection_types.single_request = match Pbrt.Decoder.key d with
      | None -> Pbrt.Decoder.malformed_variant "single_request"
      | Some (1, _) -> Connection_types.Connection_request (decode_connection_request (Pbrt.Decoder.nested d))
      | Some (2, _) -> Connection_types.Job_submission (Job_pb.decode_job_submission (Pbrt.Decoder.nested d))
      | Some (3, _) -> Connection_types.Data_retrieval_request (Data_pb.decode_data_retrieval_request (Pbrt.Decoder.nested d))
      | Some (4, _) -> Connection_types.Job_status_request (Status_pb.decode_job_status_request (Pbrt.Decoder.nested d))
      | Some (n, payload_kind) -> (
        Pbrt.Decoder.skip d payload_kind; 
        loop () 
      )
    in
    ret
  in
  loop ()

let rec decode_single_response d = 
  let rec loop () = 
    let ret:Connection_types.single_response = match Pbrt.Decoder.key d with
      | None -> Pbrt.Decoder.malformed_variant "single_response"
      | Some (1, _) -> Connection_types.Job_submission_response (Job_pb.decode_job_submission_response (Pbrt.Decoder.nested d))
      | Some (2, _) -> Connection_types.Data_retrieval_response (Data_pb.decode_data_retrieval_response (Pbrt.Decoder.nested d))
      | Some (3, _) -> Connection_types.Job_status_response (Status_pb.decode_job_status_response (Pbrt.Decoder.nested d))
      | Some (n, payload_kind) -> (
        Pbrt.Decoder.skip d payload_kind; 
        loop () 
      )
    in
    ret
  in
  loop ()

let rec encode_connection_request_status (v:Connection_types.connection_request_status) encoder =
  match v with
  | Connection_types.Heartbeat -> Pbrt.Encoder.int_as_varint (0) encoder
  | Connection_types.Close_connection -> Pbrt.Encoder.int_as_varint 1 encoder

let rec encode_connection_request (v:Connection_types.connection_request) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Varint) encoder; 
  Pbrt.Encoder.int32_as_varint v.Connection_types.user_id encoder;
  Pbrt.Encoder.key (2, Pbrt.Varint) encoder; 
  encode_connection_request_status v.Connection_types.status encoder;
  ()

let rec encode_single_request (v:Connection_types.single_request) encoder = 
  begin match v with
  | Connection_types.Connection_request x ->
    Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_connection_request x) encoder;
  | Connection_types.Job_submission x ->
    Pbrt.Encoder.key (2, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Job_pb.encode_job_submission x) encoder;
  | Connection_types.Data_retrieval_request x ->
    Pbrt.Encoder.key (3, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Data_pb.encode_data_retrieval_request x) encoder;
  | Connection_types.Job_status_request x ->
    Pbrt.Encoder.key (4, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Status_pb.encode_job_status_request x) encoder;
  end

let rec encode_single_response (v:Connection_types.single_response) encoder = 
  begin match v with
  | Connection_types.Job_submission_response x ->
    Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Job_pb.encode_job_submission_response x) encoder;
  | Connection_types.Data_retrieval_response x ->
    Pbrt.Encoder.key (2, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Data_pb.encode_data_retrieval_response x) encoder;
  | Connection_types.Job_status_response x ->
    Pbrt.Encoder.key (3, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Status_pb.encode_job_status_response x) encoder;
  end
