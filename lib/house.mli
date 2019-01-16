(** Entry point for applications using the Parliament library with a Parliament Cluster *)

exception IncorrectNumberOfOutputs
(** exception only thrown in worker mode. Parliament will check that the output of the function contains the correct number of outputs *)

val init : unit -> Door.Context.context Pervasives.ref * string
(** [init] must be called at the beginning of the application, which creates and returns the connection to the cluster.
    [init] adds a command line interface for passing in the command line cluster options 
    {b Example interface:}

    {i Parliament - A distributed general-purpose cluster-computing framework for OCaml}

    === flags ===

    - -h STRING      Cluster Hostname
    - -p INTEGER     Cluster Port
    - [-a STRING]    Cluster Authentication
    - [-d STRING]    Executable docker container
    - [-build-info]  print info about this build and exit
    - [-version]     print the version of this build and exit
    - [-help]        print this help text and exit
                 (alias: -?)

*)