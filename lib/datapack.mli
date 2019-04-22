(** Module for marshaling the polymorphic data between the workers executing the code *)

(**/**)
type datapack = {
  data: bytes array ;
}
(**/**)

val marshal : 'a -> bytes
(** [marshal item], wrapper around the OCaml marshal function which ensures [item] is marshaled the same way *)

val unmarshal : bytes -> 'a
(** [unmarshal bytes], wrapper around OCaml unmarshal function *)

val length : datapack -> int
(** [length datapack] returns the number of variables contained in datapack *)

val create : int -> datapack
(** [create length] a datapack with size [length] *)

val create_direct : bytes list -> datapack
(** [create_direct bytes_list] Create a datapack from [bytes_list] of marshaled data *)

val get_direct : datapack -> bytes list
(** [get_direct datapack] Get the underlying bytes array of the datapack as a list *)

val from_list : 'a list -> datapack
(** [from_list list] Convert [list] into a datapack *)

val single_item : 'a -> datapack
(** [single_item element] Create a single item datapack containing [element] *)

val add_item :  datapack -> 'a -> int -> unit
(** [add_item item index datapack] Add [item] to [datapack] at specific [index]. Value is marshaled, ready for transmission *)

val get_item : datapack -> int -> 'a
(** [get_item index datapack] Get [item] from [datapack] at specific [index]. Will cause value to be unmarshaled *)
