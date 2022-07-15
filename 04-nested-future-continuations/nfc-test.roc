app "nfc-test"
    packages { pf: "main.roc" }
    imports [pf.Effect.{Effect, Future}]
    provides [main] to pf

main =
    Effect.always (Done 12)
    # future <- Effect.after Effect.readData
    # Effect.always (More future (\x -> Effect.always (Done (x * x))))