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

let create n = {
  data = Array.make n empty
}

let single a = {
  data = Array.make 1 (marshal a)
}

let add x n datapack = datapack.data.(n) <- marshal x

let get x datapack = unmarshal datapack.data.(x)

let length datapack = Array.length datapack.data