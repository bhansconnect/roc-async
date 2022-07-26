app "hyper-roc"
    packages { pf: "main.roc" }
    imports [pf.Effect.{Effect, always, after}]
    provides [main] to pf

main = \req ->
    method <- Effect.method req |> after
    path <- Effect.path req |> after
    # Note: Str.split has a bug currently.
    # It returns ["/"] on the root path of "/" instead of ["", ""]
    pathList = Str.split path "/"
    resp =
        # It seems that we can't yet match on a Str.
        # hits a bug in decision_tree.rs
        # Using if instead.

        # We care about the second element for routing.
        # If it doesn't exit, we are dealing with the main root.
        route =
            when List.get pathList 1 is
                Ok x  -> x
                Err _ -> "/"
        if T method route == T Get "/" then
            {status: 200, body: "Hello, World!"}
        else if T method route == T Get "hello" then
            first = List.get pathList 2
            last = List.get pathList 3
            when T first last is
                # Roc doesn't have guards to my knowledge so adding them manually.
                T (Ok "") _ ->
                    {status: 200, body: "Hello, Mr. Nobody?"}
                T (Ok firstStr) (Ok "") ->
                    {status: 200, body: "Hello, \(firstStr)!"}
                T (Ok firstStr) (Ok lastStr) ->
                    {status: 200, body: "Hello, \(firstStr) \(lastStr)!"}
                T (Ok firstStr) _ ->
                    {status: 200, body: "Hello, \(firstStr)!"}
                _ ->
                    {status: 200, body: "Hello, Mr. Nobody?"}
        else if T method route == T Get "compute" then
            when List.get pathList 2 |> Result.try Str.toU64 is
                Ok n ->
                    {status: 200, body: Num.toStr (fibonacci n)}
                Err _ ->
                    {status: 400, body: ""}
        else
            {status: 404, body: ""}
    always resp

# This is intentionally a bad recursive fib to eat of compute time.
fibonacci : U64 -> U64
fibonacci = \n ->
    when n is
        0 -> 1
        1 -> 1
        _ -> fibonacci (n - 1) + fibonacci (n - 2)
