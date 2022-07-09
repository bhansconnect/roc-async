platform "cli"
    requires {} { main : Effect {future: Future, cont: (I32 -> I32)} }
    exposes []
    packages {}
    imports [Effect.{ Effect, Future }]
    provides [mainForHost]

mainForHost : Effect {future: Future, cont: (I32 -> I32) as Cont} as Fx
mainForHost = main
