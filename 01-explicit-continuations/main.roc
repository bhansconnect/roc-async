platform "cli"
    requires {} { main : I32 -> (I32 -> (I32 -> I32)) }
    exposes []
    packages {}
    imports []
    provides [mainForHost]

mainForHost : I32 -> (I32 -> (I32 -> I32) as Continuation) as Main
mainForHost = \x -> main x
