platform "cli"
    requires {} { main : _ }
    exposes []
    packages {}
    imports [pf.Effect.{ Effect, Future }]
    provides [mainForHost]

# TODO: everything...haha. Also, this may need to be a Task with result.
mainForHost : Str -> Effect Str
mainForHost = \x -> main x
