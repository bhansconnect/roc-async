app "ec-test"
    packages { pf: "main.roc" }
    imports []
    provides [main] to pf

main : (I32 -> (I32 -> I32))
main =
    \x ->
        z = x - 2
        \y ->
            x + y + z