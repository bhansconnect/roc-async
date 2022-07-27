platform "cli"
    requires {} { main : _ }
    exposes []
    packages {}
    imports [pf.Effect.{ Effect }]
    provides [mainForHost]

mainForHost : {} -> Effect [
    I32More ((I32 -> Effect Continuation) as I32MoreCont),
    F32More ((F32 -> Effect Continuation) as F32MoreCont),
    Done I32
    ] as Continuation
mainForHost = \{} -> main {}
