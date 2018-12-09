open Marshal
open Bytes

type datapack = {
  data: bytes array ;
}

let marshal x = Marshal.to_bytes x [Compat_32]
let unmarshal x = Marshal.from_bytes x

let create n = {
  data = Array.make n empty
}

let add x n datapack = datapack.data.(n) <- marshal x

let get x datapack = unmarshal datapack.data.(x)

let length datapack = Array.length datapack