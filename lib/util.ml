
let debug = true

(* RANDOM FUNCTIONS *)

let rec range i j = if i > j then [] else i :: (range (i+1) j)
let range (i:int32) (j:int32) = range(Int32.to_int i) (Int32.to_int j)

(* PRINTING *)

let info_print (str:string) = Printf.printf "[PARLIAMENT] %s \n" str

let error_print (str:string) = Printf.printf "[PARLIAMENT error] %s \n" str

let debug_print (str:string) =
  match debug with
    | true -> Printf.printf "[PARLIAMENT debug] %s \n" str
    | false -> ()
