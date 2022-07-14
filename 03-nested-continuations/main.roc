platform "cli"
    requires {} { main : Continuation }
    exposes []
    packages {}
    imports []
    provides [mainForHost]

mainForHost : [More ((I32 -> Continuation) as MoreCont), Done I32] as Continuation
mainForHost = main
