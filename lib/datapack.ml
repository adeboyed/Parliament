(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
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

let get_direct datapack = Array.to_list datapack.data

let from_list x = {
  data = Array.of_list (List.map marshal x)
}

let single_item a = {
  data = Array.make 1 (marshal a)
}

let add_item datapack x n  = datapack.data.(n) <- marshal x
let get_item datapack x  = unmarshal datapack.data.(x)
