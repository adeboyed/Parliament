(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

(* open Core.Std *)

(* Type *)

type connection_status = Unconnected
  | ConnectionPending
  | Connected
  | ConnectedRejection

(* class job =
  object

end *)

class parliament_connection = 
  object
    val mutable hostname: string = ""
    val mutable port: int = 0
    val mutable connection_status: connection_status = Unconnected
    val mutable user_id: string = ""

    method connnect(hn:string) (pt:int) (authentication:string) = 
      let response = Connection.send_connection_request(hn) (pt) (authentication) in
      match response.connection_accepted with
          | true -> 
            (hostname <- hn;
            port <- pt;
            connection_status <- Connected;
            user_id <- response.user_id;
            true)
          | false -> 
            (connection_status <- Unconnected;
            false)
end
