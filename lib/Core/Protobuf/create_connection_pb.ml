[@@@ocaml.warning "-27-30-39"]

type create_connection_request_mutable = {
  mutable authentication : string;
  mutable docker_name : string;
}

let default_create_connection_request_mutable () : create_connection_request_mutable = {
  authentication = "";
  docker_name = "";
}

type create_connection_response_mutable = {
  mutable user_id : string;
  mutable connection_accepted : bool;
}

let default_create_connection_response_mutable () : create_connection_response_mutable = {
  user_id = "";
  connection_accepted = false;
}

type executable_request_mutable = {
  mutable user_id : string;
  mutable executable : bytes;
}

let default_executable_request_mutable () : executable_request_mutable = {
  user_id = "";
  executable = Bytes.create 0;
}


let rec decode_create_connection_request d =
  let v = default_create_connection_request_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.authentication <- Pbrt.Decoder.string d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(create_connection_request), field(1)" pk
    | Some (2, Pbrt.Bytes) -> begin
      v.docker_name <- Pbrt.Decoder.string d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(create_connection_request), field(2)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Create_connection_types.authentication = v.authentication;
    Create_connection_types.docker_name = v.docker_name;
  } : Create_connection_types.create_connection_request)

let rec decode_create_connection_response d =
  let v = default_create_connection_response_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.user_id <- Pbrt.Decoder.string d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(create_connection_response), field(1)" pk
    | Some (2, Pbrt.Varint) -> begin
      v.connection_accepted <- Pbrt.Decoder.bool d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(create_connection_response), field(2)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Create_connection_types.user_id = v.user_id;
    Create_connection_types.connection_accepted = v.connection_accepted;
  } : Create_connection_types.create_connection_response)

let rec decode_executable_request d =
  let v = default_executable_request_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.user_id <- Pbrt.Decoder.string d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(executable_request), field(1)" pk
    | Some (2, Pbrt.Bytes) -> begin
      v.executable <- Pbrt.Decoder.bytes d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(executable_request), field(2)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Create_connection_types.user_id = v.user_id;
    Create_connection_types.executable = v.executable;
  } : Create_connection_types.executable_request)

let rec encode_create_connection_request (v:Create_connection_types.create_connection_request) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.string v.Create_connection_types.authentication encoder;
  Pbrt.Encoder.key (2, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.string v.Create_connection_types.docker_name encoder;
  ()

let rec encode_create_connection_response (v:Create_connection_types.create_connection_response) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.string v.Create_connection_types.user_id encoder;
  Pbrt.Encoder.key (2, Pbrt.Varint) encoder; 
  Pbrt.Encoder.bool v.Create_connection_types.connection_accepted encoder;
  ()

let rec encode_executable_request (v:Create_connection_types.executable_request) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.string v.Create_connection_types.user_id encoder;
  Pbrt.Encoder.key (2, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.bytes v.Create_connection_types.executable encoder;
  ()
