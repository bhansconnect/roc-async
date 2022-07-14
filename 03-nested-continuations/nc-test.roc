app "ec-test"
    packages { pf: "main.roc" }
    imports []
    provides [main] to pf

# main : I32 -> (I32 -> (I32 -> I32))
main =
    x <- More 
    a = x - 1
    y <- More 
    b = y + a
    z <- More
    Done (a + b + x + y + z)