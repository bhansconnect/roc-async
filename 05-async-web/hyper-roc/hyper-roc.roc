app "hyper-roc"
    packages { pf: "main.roc" }
    imports [pf.Effect.{Effect, always}]
    provides [main] to pf

main = \_req ->
    always "Hello, World!"