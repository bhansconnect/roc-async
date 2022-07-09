app "fc-test"
    packages { pf: "main.roc" }
    imports [pf.Effect.{Effect, Future}]
    provides [main] to pf

main : Effect {future: Future, cont: (I32 -> I32)}
main =
    future <- Effect.after Effect.readData
    Effect.always {future, cont: \x -> x * x}