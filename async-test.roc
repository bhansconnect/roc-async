app "async-test"
    packages { pf: "platform/main.roc" }
    imports [pf.Stdout, pf.Task]
    provides [main] to pf

main : Task.Task {} []
main =
    _ <- Task.await (Stdout.line "Hi")
    Task.succeed {}