platform "cli"
    requires {} { main : _ }
    exposes []
    packages {}
    imports [pf.Effect.{ Effect, Future, Request }]
    provides [mainForHost]

# TODO: everything...haha. Also, this may need to be a Task with result.
mainForHost : Request -> Effect
    [
        DBResult Future ((U64 -> Effect Continuation) as DBResultCont),
        Response { body: Str, status: U16 }
    ] as Continuation
mainForHost = \req -> main req
