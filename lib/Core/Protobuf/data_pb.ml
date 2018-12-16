[@@@ocaml.warning "-27-30-39"]

type data_retrieval_request_mutable = {
  mutable user_id : string;
  mutable job_id : int32;
}

let default_data_retrieval_request_mutable () : data_retrieval_request_mutable = {
  user_id = "";
  job_id = 0l;
}

type data_retrieval_response_mutable = {
  mutable bytes : bytes;
}

let default_data_retrieval_response_mutable () : data_retrieval_response_mutable = {
  bytes = Bytes.create 0;
}


let rec decode_data_retrieval_request d =
  let v = default_data_retrieval_request_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.user_id <- Pbrt.Decoder.string d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(data_retrieval_request), field(1)" pk
    | Some (2, Pbrt.Varint) -> begin
      v.job_id <- Pbrt.Decoder.int32_as_varint d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(data_retrieval_request), field(2)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Data_types.user_id = v.user_id;
    Data_types.job_id = v.job_id;
  } : Data_types.data_retrieval_request)

let rec decode_data_retrieval_response d =
  let v = default_data_retrieval_response_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.bytes <- Pbrt.Decoder.bytes d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(data_retrieval_response), field(1)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Data_types.bytes = v.bytes;
  } : Data_types.data_retrieval_response)

let rec encode_data_retrieval_request (v:Data_types.data_retrieval_request) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.string v.Data_types.user_id encoder;
  Pbrt.Encoder.key (2, Pbrt.Varint) encoder; 
  Pbrt.Encoder.int32_as_varint v.Data_types.job_id encoder;
  ()

let rec encode_data_retrieval_response (v:Data_types.data_retrieval_response) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.bytes v.Data_types.bytes encoder;
  ()
