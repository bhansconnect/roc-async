app "repro"
    packages { pf: "main.roc" }
    imports [pf.Effect.{always}]
    provides [main] to pf

main = \{} ->
    # Broken
    (I32More \x ->
        (F32More \y ->
            Done (x + (Num.round y)) |> always
        ) |> always
    ) |> always
    # Functional due to not using F32More
    # (I32More \x ->
    #     (I32More \y ->
    #         Done (x + y) |> always
    #     ) |> always
    # ) |> always
    # Functional due to not using I32More
    # (F32More \x ->
    #     (F32More \y ->
    #         Done (Num.round (x + y)) |> always
    #     ) |> always
    # ) |> always