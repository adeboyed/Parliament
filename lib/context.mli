(** Module that contains the abstract of a connection to a Paliament cluster: ParliamentContext *)

exception NotConnnectedException
(** exception thrown when attempting to perform a operation on a disconnected context *)

exception JobErroredException
(** exception thrown when one of the jobs throws an error *)

exception JobSubmissionException
(** exception thrown when the server rejects the jobs *)

exception InternalServerError
(** exception thrown when the server encountered an internal server error *)

type connection_status =
  | Unconnected
  | Connected
  | Disconnected

type context = {
  hostname : string;
  port : int;
  connection_status : connection_status;
  user_id : string;
  next_job : int32;
}

type status =
  | Blocked
  | Queued
  | Running
  | Completed
  | Halted
  | Cancelled

(** A type for modelling all of the different statuses of a job *)

type running_job = {
  job_id : int32;
  status : status;
}
(** A tuple for holding the ID of a job and the current status of a job*)

val connect : string -> int -> string -> string -> context Pervasives.ref
(** [connect hostname port authentication_token docker_name] initialises a connection to a Parliament cluster and then returns a reference to a context cluster. Can return an unconnected context *)

val heartbeat : context Pervasives.ref -> bool
(** [heartbeat context] sends a heartbeat request to the cluster *)

val submit : context Pervasives.ref -> Workload.workload -> running_job list option
(** [submit context workload] submits a workload to the Parliament cluster defined by the Context information *)

val job_status : context Pervasives.ref -> running_job list -> running_job list option
(** [job_status context jobs_list] sends a request to the Parliament cluster requesting an update on the jobs specified in the list *)

val all_completed : running_job list -> bool
(** [all_completed] helper function to check if {i all} of the jobs defined in the list have completed *)

val cancelled_or_halted : running_job list -> bool
(** [cancelled_or_halted] helper function to check if {i any} of the jobs defined in list have been cancelled or errored *)

val wait_until_output : context Pervasives.ref -> running_job list -> unit
(** [wait_until_output context jobs_lists] will block and return the output for a particular job once all of the jobs are succesful. Will raise exception if encounters any problem while processing *)

val output : context Pervasives.ref -> running_job list -> Datapack.datapack option
(** [output context jobs_lists] will return the output workload of the last job specified in the jobs_list *)