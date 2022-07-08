app "async-test"
    packages { pf: "platform/main.roc" }
    imports [pf.Stdout, pf.Task]
    provides [main] to pf

main : Task.Task {} []
main =
    _ <- Task.await (Stdout.line (Str.concat "Hello, allocating " "world!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"))
    _ <- Task.await (Stdout.line (Str.concat "Hello, allocating " "world2!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"))
    Task.succeed {}