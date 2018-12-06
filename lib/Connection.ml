open Unix
(* open Test *)

(* Helper functions *)
let open_connection (sockaddr:sockaddr) =
  let sock = Unix.socket Unix.PF_INET Unix.SOCK_STREAM 0 
  in try Unix.connect sock sockaddr ;
          (Unix.in_channel_of_descr sock , Unix.out_channel_of_descr sock)
    with exn -> Unix.close sock ; raise exn

let shutdown_connection (ic:in_channel) =
  Unix.shutdown (Unix.descr_of_in_channel ic) Unix.SHUTDOWN_SEND

let send_to_master client_fun (server: string) (port:int) =
  let server_addr =
    try  Unix.inet_addr_of_string server 
    with Failure("inet_addr_of_string") -> 
      try  (Unix.gethostbyname server).Unix.h_addr_list.(0) 
      with Not_found ->
        Printf.eprintf "%s : Unknown server\n" server ;
        exit 2
  in try
      let sockaddr = Unix.ADDR_INET(server_addr, port) in 
      let ic,oc = open_connection sockaddr in
      let result = client_fun ic oc
      in  shutdown_connection ic;
          result
    with Failure("int_of_string") -> Printf.eprintf "bad port number";
      exit 2

(* Actual useful functions *)

let create_connection_request_function (auth:string) (ic:in_channel) (oc:out_channel) =
  let request = Parli_core_proto.Create_connection_types.({ 
    authentication = auth
  }) in 
  let encoder = Pbrt.Encoder.create ()   (* Create a Protobuf encoder and encode value *)
  in
  Parli_core_proto.Create_connection_pb.encode_create_connection_request request encoder; 
    output_bytes oc (Pbrt.Encoder.to_bytes encoder);
    close_out oc;
  let bytes = 
      let len = in_channel_length ic in 
      let bytes = Bytes.create len in 
      really_input ic bytes 0 len; 
      close_in ic; 
      bytes 
    in 
    Parli_core_proto.Create_connection_pb.decode_create_connection_response (Pbrt.Decoder.of_bytes bytes)

let send_connection_request (hostname: string) (port: int) (auth: string)  =
  let func = create_connection_request_function
  in
    send_to_master(func(auth)) (hostname) (port)