platform "cli"
    requires {} { main : _ }
    exposes []
    packages {}
    imports [pf.Effect.{ Effect, Future }]
    provides [mainForHost]

mainForHost : I32 -> Effect [More Future ((I32 -> Effect Continuation) as MoreCont), Done I32] as Continuation
mainForHost = \x -> main x
