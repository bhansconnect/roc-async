The goal here is to be able to await in a roc function and then run a continuation.
It is essentially a mix of the previous two examples.

In this specific case the plan is to have roc call a future that sleeps for some time and then returns a number.
Roc will then take that number and simply increment it by 1 before returning to the main functions.
Many of these Roc functions should be able to run concurrently all finishing in about the same amount of time.
If async does not work correctly, we should see a tail end of functions that take longer to run.

We will use the tokio current thread executor because it only runs on a single thread.


A note on cost. Currently this is using fat pointer boxed dyn Future's.
As such, every time we call a host async effect function, we have to allocate a box on the heap.
I guess long term potentially these could be bump allocated to minimize the cost, but it is definitely an extra cost.
Though bump allocation may not be possible due to getting a task local requiring running asyncronously.
Another option is maybe thread a context through roc that contains the bump for the current task.

Long term it would be great to avoid the dyn Future all together,
but that requires explicitly wraping every future type so it is typed and sized.
I am not sure how hard this will be to do, but to my understanding it would be annoying.
With every future wrapped and knowing the exact size, we could theoretically pass them as a struct
and avoid all dynamic allocations in general.