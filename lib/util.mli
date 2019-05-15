(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
 *)

(** Module useful functions used throughout Parliament *)

val debug : bool
(** Flag of whether debug mode is enabled or not *)

val range : Int32.t -> Int32.t -> Int32.t list
(** [range lower_bound upper_bound] produces a list of containing all of the values between [lower_bound] and [upper_bound] inclusive *)

val info_print : string -> unit
(** [info_print text] prints to STDOUT a formatting info message *)

val error_print : string -> unit
(** [error_print text] prints to STDERR a formatting error message *)

val debug_print : string -> unit
(** [debug_print text] prints to STDOUT a formatting debug message *)

val minisleep : float -> unit
(** [minisleep time] causes thread to sleep for [time] seconds *)