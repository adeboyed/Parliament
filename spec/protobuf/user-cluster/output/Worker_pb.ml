[@@@ocaml.warning "-27-30-39"]

type worker_input_mutable = {
  mutable function_closure : bytes;
  mutable map_type : Worker_types.worker_input_map_type;
  mutable datapack : bytes;
}

let default_worker_input_mutable () : worker_input_mutable = {
  function_closure = Bytes.create 0;
  map_type = Worker_types.default_worker_input_map_type ();
  datapack = Bytes.create 0;
}

type worker_output_mutable = {
  mutable datapack : bytes;
}

let default_worker_output_mutable () : worker_output_mutable = {
  datapack = Bytes.create 0;
}


let rec decode_worker_input_map_type d = 
  match Pbrt.Decoder.int_as_varint d with
  | 0 -> (Worker_types.Single_in_variable_out:Worker_types.worker_input_map_type)
  | 1 -> (Worker_types.Single_in_single_out:Worker_types.worker_input_map_type)
  | 2 -> (Worker_types.Variable_in_variable_out:Worker_types.worker_input_map_type)
  | _ -> Pbrt.Decoder.malformed_variant "worker_input_map_type"

let rec decode_worker_input d =
  let v = default_worker_input_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.function_closure <- Pbrt.Decoder.bytes d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(worker_input), field(1)" pk
    | Some (2, Pbrt.Varint) -> begin
      v.map_type <- decode_worker_input_map_type d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(worker_input), field(2)" pk
    | Some (3, Pbrt.Bytes) -> begin
      v.datapack <- Pbrt.Decoder.bytes d;
    end
    | Some (3, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(worker_input), field(3)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Worker_types.function_closure = v.function_closure;
    Worker_types.map_type = v.map_type;
    Worker_types.datapack = v.datapack;
  } : Worker_types.worker_input)

let rec decode_worker_output d =
  let v = default_worker_output_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (3, Pbrt.Bytes) -> begin
      v.datapack <- Pbrt.Decoder.bytes d;
    end
    | Some (3, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(worker_output), field(3)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Worker_types.datapack = v.datapack;
  } : Worker_types.worker_output)

let rec encode_worker_input_map_type (v:Worker_types.worker_input_map_type) encoder =
  match v with
  | Worker_types.Single_in_variable_out -> Pbrt.Encoder.int_as_varint (0) encoder
  | Worker_types.Single_in_single_out -> Pbrt.Encoder.int_as_varint 1 encoder
  | Worker_types.Variable_in_variable_out -> Pbrt.Encoder.int_as_varint 2 encoder

let rec encode_worker_input (v:Worker_types.worker_input) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.bytes v.Worker_types.function_closure encoder;
  Pbrt.Encoder.key (2, Pbrt.Varint) encoder; 
  encode_worker_input_map_type v.Worker_types.map_type encoder;
  Pbrt.Encoder.key (3, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.bytes v.Worker_types.datapack encoder;
  ()

let rec encode_worker_output (v:Worker_types.worker_output) encoder = 
  Pbrt.Encoder.key (3, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.bytes v.Worker_types.datapack encoder;
  ()
