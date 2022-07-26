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
    # We care about the second element for routing.
    # If it doesn't exit, we are dealing with the main root.
    route =
        when List.get pathList 1 is
            Ok x  -> x
            Err _ -> "/"
    # It seems that we can't yet match on a Str.
    # hits a bug in decision_tree.rs
    # Using if instead.
    if T method route == T Get "/" then
        Response {status: 200, body: "Hello, World!"} |> always
    else if T method route == T Get "hello" then
        first = List.get pathList 2
        last = List.get pathList 3
        when T first last is
            # Roc doesn't have guards to my knowledge so adding them manually.
            T (Ok "") _ ->
                Response {status: 200, body: "Hello, Mr. Nobody?"} |> always
            T (Ok firstStr) (Ok "") ->
                Response {status: 200, body: "Hello, \(firstStr)!"} |> always
            T (Ok firstStr) (Ok lastStr) ->
                Response {status: 200, body: "Hello, \(firstStr) \(lastStr)!"} |> always
            T (Ok firstStr) _ ->
                Response {status: 200, body: "Hello, \(firstStr)!"} |> always
            _ ->
                Response {status: 200, body: "Hello, Mr. Nobody?"} |> always
    else if T method route == T Get "compute" then
        when List.get pathList 2 |> Result.try Str.toU64 is
            Ok n ->
                Response {status: 200, body: Num.toStr (fibonacci n)} |> always
            Err _ ->
                Response {status: 400, body: ""} |> always
    else if T method route == T Get "sleep" then
        delayMSResult = List.get pathList 2 |> Result.try Str.toU64
        repsResult = List.get pathList 3 |> Result.try Str.toU64
        when T delayMSResult repsResult is
            # The next 2 cases should be unified, but the matching doesn't seem to work.
            T (Ok delayMS) (Ok 1) ->
                always (DBRequest delayMS \res ->
                    resStr = Num.toStr res
                    Response {status: 200, body: "\(resStr) Nap Completed"} |> always
                )
            T (Ok delayMS) (Err _) ->
                always (DBRequest delayMS \res ->
                    resStr = Num.toStr res
                    Response {status: 200, body: "\(resStr) Nap Completed"} |> always
                )
            T (Ok _delayMS) (Ok _reps) ->
                # TODO: Fix this
                # sleepRepsHelper delayMS reps 0
                Response {status: 500, body: ""} |> always
            _ ->
                Response {status: 400, body: ""} |> always
    else if T method route == T Get "dup" then
        when List.get pathList 2 |> Result.try Str.toNat is
            Ok n ->
                always (LoadBody \res ->
                    when res is
                        Ok body ->
                            dup = Str.repeat body n
                            Response {status: 200, body: "\(dup)"} |> always
                        _ ->
                            Response {status: 400, body: ""} |> always
                )
            Err _ ->
                Response {status: 400, body: ""} |> always
    else
        Response {status: 404, body: ""} |> always

# TODO: why does Roc not like the recursion here.
# Can I write it differently to make it work?
# Are recursive functions with effects not allowed?
# sleepRepsHelper = \delayMS, reps, sum ->
#     if reps > 0 then
#         future <- Effect.fakeDBCall delayMS |> after
#         DBResult future (\res -> sleepRepsHelper delayMS (reps - 1) (sum + res)) |> always
#     else
#         sumStr = Num.toStr sum
#         Response {status: 200, body: "\(sumStr) Naps Completed"} |> always


# This is intentionally a bad recursive fib to eat of compute time.
fibonacci : U64 -> U64
fibonacci = \n ->
    when n is
        0 -> 1
        1 -> 1
        _ -> fibonacci (n - 1) + fibonacci (n - 2)
