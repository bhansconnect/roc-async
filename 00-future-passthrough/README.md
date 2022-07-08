This is the starting test case with a future.
All it does it have roc call a function that returns a future.
That future is then returned to the host and run by the default async executor.

It shows that roc can deal with futures without really knowing what they are.

run with `roc fp-test.roc`