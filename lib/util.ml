

let debug = true

let info_print (str:string) = Printf.printf "[PARLIAMENT] %s \n" str

let error_print (str:string) = Printf.printf "[PARLIAMENT error] %s \n" str

let debug_print (str:string) =
  match debug with
    | true -> Printf.printf "[PARLIAMENT debug] %s \n" str
    | false -> ()