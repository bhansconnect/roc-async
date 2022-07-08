app "ec-test"
    packages { pf: "main.roc" }
    imports []
    provides [main] to pf

main : I32 -> (I32 -> (I32 -> I32))
main =
    \x ->
        a = x - 1
        \y ->
            b = y + a
            \z ->
                a + b + x + y + z