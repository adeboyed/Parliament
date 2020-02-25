# Parliament

A Cambridge Computer Science Part II dissertation building a general-purpose cluster computing framework for OCaml.

Parliament is a data-parallel distributed library, where the architecture encourages developers to write parallelised workloads, while not restricting developers from writing whatever function they want.

![Parliament Architecture](/architecture.png?raw=true)

There are 3 main parts to this system:
1. **Prime Minister**: Cluster master, Rust executable that communicates between the users and workers, handles fault-tolerance and task assignment.
2. **Parliament**: OCaml library, which handles data marshalling and communication with the cluster.
3. **Member of Parliament**: Cluster worker, Rust executable that communicates with the Prime Minister, that wraps around the OCaml executable and runs closures in the OCaml executable.


More information can be found in my [dissertation](dissertation.pdf).
