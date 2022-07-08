platform "cli"
    requires {} { main : Effect Future }
    exposes []
    packages {}
    imports [Effect.{ Effect, Future }]
    provides [mainForHost]

mainForHost : Effect Future as Fx
mainForHost = main
