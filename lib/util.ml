(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

let debug = true

(* RANDOM FUNCTIONS *)

(* let rec range i j = if i > j then [] else i :: (range (i+1) j) *)
let rec range i j = if (Int32.compare i j) > 0 then [] else i :: (range (Int32.succ i) j);;

(* PRINTING *)

let info_print (str:string) = Printf.printf "[PARLIAMENT] %s \n%!" str

let error_print (str:string) = Printf.printf "[PARLIAMENT error] %s \n%!" str

let debug_print (str:string) =
  match debug with
  | true -> Printf.printf "[PARLIAMENT debug] %s \n%!" str
  | false -> ()

let minisleep (sec: float) =
  ignore (Unix.select [] [] [] sec)