The goal here is to be able to await in a roc function and then run a continuation.
It is essentially a mix of the previous two examples.

In this specific case the plan is to have roc call a future that sleeps for some time and then returns a number.
Roc will then take that number and simply increment it by 1 before returning to the main functions.
Many of these Roc functions should be able to run concurrently all finishing in about the same amount of time.
If async does not work correctly, we should see a tail end of functions that take longer to run.

We will use the tokio current thread executor because it only runs on a single thread.