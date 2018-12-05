(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

(* open Core.Std *)

(* Type *)

type connection_status = Unconnected
  | ConnectionPending
  | Connected


(* Variables *)

class parliament_connection = 
  object
    val mutable hostname: string = ""
    val mutable port: string = ""
    val mutable connection_status: connection_status = Unconnected

    method connnect(hn, pt) = 
      hostname <- hn;
      port <- pt;




end