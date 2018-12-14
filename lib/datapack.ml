(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 David Adeboye <doaa2@cl.cam.ac.uk>
 *)

open Marshal
open Bytes

type datapack = {
  data: bytes array ;
}

let marshal x = Marshal.to_bytes x [Compat_32]
let unmarshal x = Marshal.from_bytes x 0

let length datapack = Array.length datapack.data

let create n = {
  data = Array.make n empty
}

let create_direct x = {
  data = Array.of_list x
}

let single_item a = {
  data = Array.make 1 (marshal a)
}

let add_single_item x n datapack = datapack.data.(n) <- marshal x
let get_single_item x datapack = unmarshal datapack.data.(x)
