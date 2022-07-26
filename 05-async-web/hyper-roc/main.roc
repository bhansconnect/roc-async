platform "cli"
    requires {} { main : _ }
    exposes []
    packages {}
    imports [pf.Effect.{ Effect, Future, Request }]
    provides [mainForHost]

mainForHost : Request -> Effect
    [
        DBRequest U64 ((U64 -> Effect Continuation) as DBRequestCont),
        # It seems having 2 closures and using them leads a bug with infinite recursion.
        # LoadBody ((Result Str {} -> Effect Continuation) as LoadBodyCont),
        Response { body: Str, status: U16 }
    ] as Continuation
mainForHost = \req -> main req
