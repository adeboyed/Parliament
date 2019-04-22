[@@@ocaml.warning "-27-30-39"]

type connection_request_mutable = {
  mutable user_id : string;
  mutable action : Connection_types.connection_request_action;
}

let default_connection_request_mutable () : connection_request_mutable = {
  user_id = "";
  action = Connection_types.default_connection_request_action ();
}

type connection_response_mutable = {
  mutable request_accepted : bool;
}

let default_connection_response_mutable () : connection_response_mutable = {
  request_accepted = false;
}

type server_message_mutable = {
  mutable action : Connection_types.server_message_action;
}

let default_server_message_mutable () : server_message_mutable = {
  action = Connection_types.default_server_message_action ();
}


let rec decode_connection_request_action d = 
  match Pbrt.Decoder.int_as_varint d with
  | 0 -> (Connection_types.Heartbeat:Connection_types.connection_request_action)
  | 1 -> (Connection_types.Close_connection:Connection_types.connection_request_action)
  | _ -> Pbrt.Decoder.malformed_variant "connection_request_action"

let rec decode_connection_request d =
  let v = default_connection_request_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.user_id <- Pbrt.Decoder.string d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(connection_request), field(1)" pk
    | Some (2, Pbrt.Varint) -> begin
      v.action <- decode_connection_request_action d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(connection_request), field(2)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Connection_types.user_id = v.user_id;
    Connection_types.action = v.action;
  } : Connection_types.connection_request)

let rec decode_connection_response d =
  let v = default_connection_response_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Varint) -> begin
      v.request_accepted <- Pbrt.Decoder.bool d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(connection_response), field(1)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Connection_types.request_accepted = v.request_accepted;
  } : Connection_types.connection_response)

let rec decode_server_message_action d = 
  match Pbrt.Decoder.int_as_varint d with
  | 0 -> (Connection_types.User_timeout:Connection_types.server_message_action)
  | 1 -> (Connection_types.Missing_jobs:Connection_types.server_message_action)
  | 2 -> (Connection_types.Internal_server_error:Connection_types.server_message_action)
  | _ -> Pbrt.Decoder.malformed_variant "server_message_action"

let rec decode_server_message d =
  let v = default_server_message_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Varint) -> begin
      v.action <- decode_server_message_action d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(server_message), field(1)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Connection_types.action = v.action;
  } : Connection_types.server_message)

let rec decode_single_user_request d = 
  let rec loop () = 
    let ret:Connection_types.single_user_request = match Pbrt.Decoder.key d with
      | None -> Pbrt.Decoder.malformed_variant "single_user_request"
      | Some (1, _) -> Connection_types.Create_connection_request (Create_connection_pb.decode_create_connection_request (Pbrt.Decoder.nested d))
      | Some (2, _) -> Connection_types.Connection_request (decode_connection_request (Pbrt.Decoder.nested d))
      | Some (3, _) -> Connection_types.Job_submission (Job_pb.decode_job_submission (Pbrt.Decoder.nested d))
      | Some (4, _) -> Connection_types.Data_retrieval_request (Data_pb.decode_data_retrieval_request (Pbrt.Decoder.nested d))
      | Some (5, _) -> Connection_types.Job_status_request (Status_pb.decode_job_status_request (Pbrt.Decoder.nested d))
      | Some (6, _) -> Connection_types.Executable_request (Create_connection_pb.decode_executable_request (Pbrt.Decoder.nested d))
      | Some (n, payload_kind) -> (
        Pbrt.Decoder.skip d payload_kind; 
        loop () 
      )
    in
    ret
  in
  loop ()

let rec decode_single_user_response d = 
  let rec loop () = 
    let ret:Connection_types.single_user_response = match Pbrt.Decoder.key d with
      | None -> Pbrt.Decoder.malformed_variant "single_user_response"
      | Some (1, _) -> Connection_types.Create_connection_response (Create_connection_pb.decode_create_connection_response (Pbrt.Decoder.nested d))
      | Some (2, _) -> Connection_types.Job_submission_response (Job_pb.decode_job_submission_response (Pbrt.Decoder.nested d))
      | Some (3, _) -> Connection_types.Data_retrieval_response (Data_pb.decode_data_retrieval_response (Pbrt.Decoder.nested d))
      | Some (4, _) -> Connection_types.Job_status_response (Status_pb.decode_job_status_response (Pbrt.Decoder.nested d))
      | Some (5, _) -> Connection_types.Connection_response (decode_connection_response (Pbrt.Decoder.nested d))
      | Some (6, _) -> Connection_types.Server_message (decode_server_message (Pbrt.Decoder.nested d))
      | Some (n, payload_kind) -> (
        Pbrt.Decoder.skip d payload_kind; 
        loop () 
      )
    in
    ret
  in
  loop ()

let rec encode_connection_request_action (v:Connection_types.connection_request_action) encoder =
  match v with
  | Connection_types.Heartbeat -> Pbrt.Encoder.int_as_varint (0) encoder
  | Connection_types.Close_connection -> Pbrt.Encoder.int_as_varint 1 encoder

let rec encode_connection_request (v:Connection_types.connection_request) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.string v.Connection_types.user_id encoder;
  Pbrt.Encoder.key (2, Pbrt.Varint) encoder; 
  encode_connection_request_action v.Connection_types.action encoder;
  ()

let rec encode_connection_response (v:Connection_types.connection_response) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Varint) encoder; 
  Pbrt.Encoder.bool v.Connection_types.request_accepted encoder;
  ()

let rec encode_server_message_action (v:Connection_types.server_message_action) encoder =
  match v with
  | Connection_types.User_timeout -> Pbrt.Encoder.int_as_varint (0) encoder
  | Connection_types.Missing_jobs -> Pbrt.Encoder.int_as_varint 1 encoder
  | Connection_types.Internal_server_error -> Pbrt.Encoder.int_as_varint 2 encoder

let rec encode_server_message (v:Connection_types.server_message) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Varint) encoder; 
  encode_server_message_action v.Connection_types.action encoder;
  ()

let rec encode_single_user_request (v:Connection_types.single_user_request) encoder = 
  begin match v with
  | Connection_types.Create_connection_request x ->
    Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Create_connection_pb.encode_create_connection_request x) encoder;
  | Connection_types.Connection_request x ->
    Pbrt.Encoder.key (2, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_connection_request x) encoder;
  | Connection_types.Job_submission x ->
    Pbrt.Encoder.key (3, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Job_pb.encode_job_submission x) encoder;
  | Connection_types.Data_retrieval_request x ->
    Pbrt.Encoder.key (4, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Data_pb.encode_data_retrieval_request x) encoder;
  | Connection_types.Job_status_request x ->
    Pbrt.Encoder.key (5, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Status_pb.encode_job_status_request x) encoder;
  | Connection_types.Executable_request x ->
    Pbrt.Encoder.key (6, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Create_connection_pb.encode_executable_request x) encoder;
  end

let rec encode_single_user_response (v:Connection_types.single_user_response) encoder = 
  begin match v with
  | Connection_types.Create_connection_response x ->
    Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Create_connection_pb.encode_create_connection_response x) encoder;
  | Connection_types.Job_submission_response x ->
    Pbrt.Encoder.key (2, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Job_pb.encode_job_submission_response x) encoder;
  | Connection_types.Data_retrieval_response x ->
    Pbrt.Encoder.key (3, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Data_pb.encode_data_retrieval_response x) encoder;
  | Connection_types.Job_status_response x ->
    Pbrt.Encoder.key (4, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (Status_pb.encode_job_status_response x) encoder;
  | Connection_types.Connection_response x ->
    Pbrt.Encoder.key (5, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_connection_response x) encoder;
  | Connection_types.Server_message x ->
    Pbrt.Encoder.key (6, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_server_message x) encoder;
  end
