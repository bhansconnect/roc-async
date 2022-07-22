app "nfc-test"
    packages { pf: "main.roc" }
    imports [pf.Effect.{Effect, Future, always, after}]
    provides [main] to pf

# This breaks things if I put it in the effect module.
# Putting it here instead.
effectReadData = \cont ->
    future <- after Effect.readData
    always (More future cont)

main = \x ->
    when x is
        0 ->
            a <- effectReadData
            b <- effectReadData
            c <- effectReadData
            always (Done (a*b*c))
        1 ->
            y <- effectReadData
            z <- effectReadData
            always (Done (y * z))
        2 ->
            y <- effectReadData
            always (Done (y))
        _ ->
            # This should be impossible.
            # Panic
            bad : U8
            bad = 0 - 1
            Effect.always (Done -1)
