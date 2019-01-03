(** Module for marshalling the polymorphic data between the workers executing the code *)

(**/**)
type datapack = {
  data: bytes array ;
}
(**/**)

val marshal : 'a -> bytes
(** [marshal item], wrapper around the OCaml marshal function which ensures [item] is marshalled the same way *)

val unmarshal : bytes -> 'a
(** [unmarshal bytes], wrapper around OCaml unmarshal function *)

val length : datapack -> int
(** [length datapack] returns the number of variables contained in datapack *)

val create : int -> datapack
(** [create length] a datapack with size [length] *)

val create_direct : bytes list -> datapack
(** [create_direct bytes_list] Create a datapack from [bytes_list] of marshalled data*)

val get_direct : datapack -> bytes list
(** [get_direct datapack] *)

val single_item : 'a -> datapack
(** [single_item element] Create a single item datapack containing [element] *)

val add_item :  datapack -> 'a -> int -> unit
(** [add_item item index datapack] Add [item] to [datapack] at specific [index]. Value is marshalled, ready for transmision *)

val get_item : datapack -> int -> 'a
(** [get_item index datapack] Get [item] from [datapack] at specific [index]. Will cause value to be unmarshalled *)
