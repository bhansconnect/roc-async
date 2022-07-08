app "fp-test"
    packages { pf: "main.roc" }
    imports [pf.Effect.{Effect, Future}]
    provides [main] to pf

main : Effect Future
main =
    future <- Effect.after Effect.sleep
    Effect.always future