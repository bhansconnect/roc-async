platform "cli"
    requires {} { main : _ }
    exposes []
    packages {}
    imports [pf.Effect.{ Effect, Future, Request }]
    provides [mainForHost]

# TODO: everything...haha. Also, this may need to be a Task with result.
mainForHost : Request -> Effect { body: Str, status: U16 } as Main
mainForHost = \req -> main req
