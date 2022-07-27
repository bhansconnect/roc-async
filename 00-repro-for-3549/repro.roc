app "repro"
    packages { pf: "main.roc" }
    imports [pf.Effect.{always}]
    provides [main] to pf

i32More = \cont ->
    I32More cont |> always

f32More = \cont ->
    F32More cont |> always

main = \{} ->
    x <- i32More
    y <- f32More
    a <- i32More
    z <- f32More
    Done (x + a + (Num.round (y + z))) |> always