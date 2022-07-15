platform "cli"
    requires {} { main : _ }
    exposes []
    packages {}
    imports [Effect.{ Effect, Future }]
    provides [mainForHost]

mainForHost : Effect [More Future ((I32 -> Effect Continuation) as MoreCont), Done I32] as Continuation
mainForHost = main
