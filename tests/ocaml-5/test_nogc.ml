external mmtk_alloc_print : int -> unit = "caml_mmtk5_alloc_print"

let () =
  Printf.printf "MMTk OCaml5 NoGC test\n%!";
  for wosize = 1 to 5 do
    mmtk_alloc_print wosize
  done;
  Printf.printf "All allocations OK\n%!"
