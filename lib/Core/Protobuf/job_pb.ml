[@@@ocaml.warning "-27-30-39"]

type input_action_mutable = {
  mutable data_loc_in : bytes;
}

let default_input_action_mutable () : input_action_mutable = {
  data_loc_in = Bytes.create 0;
}

type map_action_mutable = {
  mutable data_loc_in : int32 list;
  mutable function_name : string;
  mutable data_loc_out : int32;
}

let default_map_action_mutable () : map_action_mutable = {
  data_loc_in = [];
  function_name = "";
  data_loc_out = 0l;
}

type output_action_mutable = {
  mutable data_out : int32;
}

let default_output_action_mutable () : output_action_mutable = {
  data_out = 0l;
}

type job_mutable = {
  mutable job_id : int32;
  mutable action : Job_types.job_action;
}

let default_job_mutable () : job_mutable = {
  job_id = 0l;
  action = Job_types.Input (Job_types.default_input_action ());
}

type job_submission_mutable = {
  mutable user_id : string;
  mutable function_name : string;
  mutable jobs : Job_types.job list;
}

let default_job_submission_mutable () : job_submission_mutable = {
  user_id = "";
  function_name = "";
  jobs = [];
}

type job_submission_response_mutable = {
  mutable job_accepted : bool;
}

let default_job_submission_response_mutable () : job_submission_response_mutable = {
  job_accepted = false;
}


let rec decode_input_action d =
  let v = default_input_action_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.data_loc_in <- Pbrt.Decoder.bytes d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(input_action), field(1)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Job_types.data_loc_in = v.data_loc_in;
  } : Job_types.input_action)

let rec decode_map_action d =
  let v = default_map_action_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
      v.data_loc_in <- List.rev v.data_loc_in;
    ); continue__ := false
    | Some (1, Pbrt.Varint) -> begin
      v.data_loc_in <- (Pbrt.Decoder.int32_as_varint d) :: v.data_loc_in;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(map_action), field(1)" pk
    | Some (2, Pbrt.Bytes) -> begin
      v.function_name <- Pbrt.Decoder.string d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(map_action), field(2)" pk
    | Some (3, Pbrt.Varint) -> begin
      v.data_loc_out <- Pbrt.Decoder.int32_as_varint d;
    end
    | Some (3, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(map_action), field(3)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Job_types.data_loc_in = v.data_loc_in;
    Job_types.function_name = v.function_name;
    Job_types.data_loc_out = v.data_loc_out;
  } : Job_types.map_action)

let rec decode_output_action d =
  let v = default_output_action_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (3, Pbrt.Varint) -> begin
      v.data_out <- Pbrt.Decoder.int32_as_varint d;
    end
    | Some (3, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(output_action), field(3)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Job_types.data_out = v.data_out;
  } : Job_types.output_action)

let rec decode_job_action d = 
  let rec loop () = 
    let ret:Job_types.job_action = match Pbrt.Decoder.key d with
      | None -> Pbrt.Decoder.malformed_variant "job_action"
      | Some (4, _) -> Job_types.Input (decode_input_action (Pbrt.Decoder.nested d))
      | Some (5, _) -> Job_types.Map (decode_map_action (Pbrt.Decoder.nested d))
      | Some (6, _) -> Job_types.Output (decode_output_action (Pbrt.Decoder.nested d))
      | Some (n, payload_kind) -> (
        Pbrt.Decoder.skip d payload_kind; 
        loop () 
      )
    in
    ret
  in
  loop ()

and decode_job d =
  let v = default_job_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Varint) -> begin
      v.job_id <- Pbrt.Decoder.int32_as_varint d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job), field(1)" pk
    | Some (4, Pbrt.Bytes) -> begin
      v.action <- Job_types.Input (decode_input_action (Pbrt.Decoder.nested d));
    end
    | Some (4, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job), field(4)" pk
    | Some (5, Pbrt.Bytes) -> begin
      v.action <- Job_types.Map (decode_map_action (Pbrt.Decoder.nested d));
    end
    | Some (5, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job), field(5)" pk
    | Some (6, Pbrt.Bytes) -> begin
      v.action <- Job_types.Output (decode_output_action (Pbrt.Decoder.nested d));
    end
    | Some (6, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job), field(6)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Job_types.job_id = v.job_id;
    Job_types.action = v.action;
  } : Job_types.job)

let rec decode_job_submission d =
  let v = default_job_submission_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
      v.jobs <- List.rev v.jobs;
    ); continue__ := false
    | Some (1, Pbrt.Bytes) -> begin
      v.user_id <- Pbrt.Decoder.string d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_submission), field(1)" pk
    | Some (2, Pbrt.Bytes) -> begin
      v.function_name <- Pbrt.Decoder.string d;
    end
    | Some (2, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_submission), field(2)" pk
    | Some (3, Pbrt.Bytes) -> begin
      v.jobs <- (decode_job (Pbrt.Decoder.nested d)) :: v.jobs;
    end
    | Some (3, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_submission), field(3)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Job_types.user_id = v.user_id;
    Job_types.function_name = v.function_name;
    Job_types.jobs = v.jobs;
  } : Job_types.job_submission)

let rec decode_job_submission_response d =
  let v = default_job_submission_response_mutable () in
  let continue__= ref true in
  while !continue__ do
    match Pbrt.Decoder.key d with
    | None -> (
    ); continue__ := false
    | Some (1, Pbrt.Varint) -> begin
      v.job_accepted <- Pbrt.Decoder.bool d;
    end
    | Some (1, pk) -> 
      Pbrt.Decoder.unexpected_payload "Message(job_submission_response), field(1)" pk
    | Some (_, payload_kind) -> Pbrt.Decoder.skip d payload_kind
  done;
  ({
    Job_types.job_accepted = v.job_accepted;
  } : Job_types.job_submission_response)

let rec encode_input_action (v:Job_types.input_action) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.bytes v.Job_types.data_loc_in encoder;
  ()

let rec encode_map_action (v:Job_types.map_action) encoder = 
  List.iter (fun x -> 
    Pbrt.Encoder.key (1, Pbrt.Varint) encoder; 
    Pbrt.Encoder.int32_as_varint x encoder;
  ) v.Job_types.data_loc_in;
  Pbrt.Encoder.key (2, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.string v.Job_types.function_name encoder;
  Pbrt.Encoder.key (3, Pbrt.Varint) encoder; 
  Pbrt.Encoder.int32_as_varint v.Job_types.data_loc_out encoder;
  ()

let rec encode_output_action (v:Job_types.output_action) encoder = 
  Pbrt.Encoder.key (3, Pbrt.Varint) encoder; 
  Pbrt.Encoder.int32_as_varint v.Job_types.data_out encoder;
  ()

let rec encode_job_action (v:Job_types.job_action) encoder = 
  begin match v with
  | Job_types.Input x ->
    Pbrt.Encoder.key (4, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_input_action x) encoder;
  | Job_types.Map x ->
    Pbrt.Encoder.key (5, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_map_action x) encoder;
  | Job_types.Output x ->
    Pbrt.Encoder.key (6, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_output_action x) encoder;
  end

and encode_job (v:Job_types.job) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Varint) encoder; 
  Pbrt.Encoder.int32_as_varint v.Job_types.job_id encoder;
  begin match v.Job_types.action with
  | Job_types.Input x ->
    Pbrt.Encoder.key (4, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_input_action x) encoder;
  | Job_types.Map x ->
    Pbrt.Encoder.key (5, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_map_action x) encoder;
  | Job_types.Output x ->
    Pbrt.Encoder.key (6, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_output_action x) encoder;
  end;
  ()

let rec encode_job_submission (v:Job_types.job_submission) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.string v.Job_types.user_id encoder;
  Pbrt.Encoder.key (2, Pbrt.Bytes) encoder; 
  Pbrt.Encoder.string v.Job_types.function_name encoder;
  List.iter (fun x -> 
    Pbrt.Encoder.key (3, Pbrt.Bytes) encoder; 
    Pbrt.Encoder.nested (encode_job x) encoder;
  ) v.Job_types.jobs;
  ()

let rec encode_job_submission_response (v:Job_types.job_submission_response) encoder = 
  Pbrt.Encoder.key (1, Pbrt.Varint) encoder; 
  Pbrt.Encoder.bool v.Job_types.job_accepted encoder;
  ()
