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
    "Allow VariableInput jobs" >:: (fun _ ->
      let single_datapack = Datapack.create 2 in
      let example_func wl = wl in 
      let workload = add_all (input single_datapack) 
          [SingleInSingleOut(example_func); VariableInSingleOut(example_func)] 
      in
      let validate_call() = validate workload in 
      try ( validate_call() )
      with _ -> assert_failure "Exception thrown on correct input"
    );
    "Don't allow increase of Variable jobs" >:: (fun _ ->
      let single_datapack = Datapack.create 2 in
      let example_func wl = wl in 
      let workload = add_all (input single_datapack) 
          [SingleInVariableOut(example_func); VariableInSingleOut(example_func)] 
      in
      let validate_call() = validate workload in 
      assert_raises IncorrectFormulationOfStages validate_call
    );
  ]

let _ = run_test_tt_main suite