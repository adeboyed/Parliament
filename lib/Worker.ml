(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

 open Core.Std

 (* Variables *)



 (* Load in data *)
 
 

 (* let create () = { no_vars = 0; vars = [] } *)

 let loadvars () = 
    let no_of_vars = input_binary_int stdin in
    match no_of_vars with
      0 -> print_string "hello";
    | x -> print_int x;




    