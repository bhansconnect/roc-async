app "hyper-roc"
    packages { pf: "main.roc" }
    imports [pf.Effect.{Effect, always, after}]
    provides [main] to pf

main = \req ->
    method <- Effect.method req |> after
    resp =
        when method is
            Get ->
                {status: 200, body: "Hello, World!"}
            _ ->
                {status: 404, body: ""}
    always resp