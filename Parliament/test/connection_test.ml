(*
 * Parliament - A distributed general-purpose cluster-computing framework for OCaml
 * Copyright (c) 2018-2019 [Name REDACTED] [Email REDACTED]
 *)

open Parliament
open Parliament.Door.Workload

open Unix
open OUnit

open Mock

let suite =
  "Workload Suite" >::: [
    "Test opens connection to correct socket" >:: (fun _ ->

    let input_fd = run_connect_to_correct_server_test 1050 in
    let _ = House.init() in
    match (try 
    let input_channel = in_channel_of_descr input_fd in
      let success : bool = input_value input_channel in
      success
    with _ -> false)
      with
      false -> assert_failure "Did not connect"
      | _ -> ()
    )
  ]

let _ = run_test_tt_main suite