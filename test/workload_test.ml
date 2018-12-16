open Parliament
open Parliament.Door.Workload

open OUnit

let suite =
  "Workload Suite" >::: [
    "MustHaveAtLeastOneJob exception" >:: (fun _ ->
        let single_datapack = Datapack.create 1 in
        let validate_call() = validate (input single_datapack) in 
        assert_raises MustHaveAtLeastOneJob validate_call
      );
    "MaxOneInputValue exception" >:: (fun _ ->
        let multi_datapack = Datapack.create 4 in
        let example_func wl = wl in 
        let workload = add (input multi_datapack) (SingleInSingleOut(example_func)) in
        let validate_call() = validate workload in 
        assert_raises MaxOneInputValue validate_call
      );
    "LastJobMustBeSingleOutput exception" >:: (fun _ ->
        let single_datapack = Datapack.create 1 in
        let example_func wl = wl in 
        let workload = add (input single_datapack) (SingleInVariableOut(example_func)) in
        let validate_call() = validate workload in 
        assert_raises LastJobMustBeSingleOutput validate_call
      );
    "IncorrectFormulationOfStages exception #1" >:: (fun _ ->
        let single_datapack = Datapack.create 1 in
        let example_func wl = wl in 
        let workload = add (input single_datapack) (VariableInSingleOut(example_func)) in
        let validate_call() = validate workload in 
        assert_raises IncorrectFormulationOfStages validate_call
      );
    "IncorrectFormulationOfStages exception #2" >:: (fun _ ->
        let single_datapack = Datapack.create 1 in
        let example_func wl = wl in 
        let workload = add_all (input single_datapack) 
            [SingleInSingleOut(example_func); VariableInSingleOut(example_func)] 
        in
        let validate_call() = validate workload in 
        assert_raises IncorrectFormulationOfStages validate_call
      );
    "IncorrectFormulationOfStages exception #3" >:: (fun _ ->
        let single_datapack = Datapack.create 1 in
        let example_func wl = wl in 
        let workload = add_all (input single_datapack) 
            [SingleInSingleOut(example_func); VariableInSingleOut(example_func); SingleInSingleOut(example_func); VariableInSingleOut(example_func)] 
        in
        let validate_call() = validate workload in 
        assert_raises IncorrectFormulationOfStages validate_call
      );
    "IncorrectFormulationOfStages exception #4" >:: (fun _ ->
        let single_datapack = Datapack.create 1 in
        let example_func wl = wl in 
        let workload = add_all (input single_datapack) 
            [SingleInSingleOut(example_func); VariableInSingleOut(example_func); SingleInSingleOut(example_func); VariableInSingleOut(example_func)] 
        in
        let validate_call() = build workload Int32.one in 
        assert_raises IncorrectFormulationOfStages validate_call
      );
    "Validates correct order of jobs" >:: (fun _ ->
        let single_datapack = Datapack.create 1 in
        let example_func wl = wl in 
        let workload = add_all (input single_datapack) 
            [SingleInVariableOut(example_func); SingleInSingleOut(example_func); VariableInSingleOut(example_func)] 
        in
        let validate_call() = validate workload in 
        try ( validate_call() )
        with _ -> assert_failure "Exception thrown on correct input"
      );
    (* "Builds jobs in the correct order" >:: (fun _ ->
        let multi_datapack = Datapack.create 4 in
        let example_func wl = wl in 
        let workload = add_all (input multi_datapack) 
            [SingleInVariableOut(example_func); SingleInSingleOut(example_func); VariableInSingleOut(example_func)]
        in
        let jobs = build workload Int32.one in 
        match jobs with
          [{ map_type}] -> ()
        | _ -> assert_failure "Incorrect "
       ); *)
  ]

let _ = run_test_tt_main suite