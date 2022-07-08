On theoretical goal of all of this is to have an arena for each async task.
This is totally doable. tokio specifically has `tokio::task_local!` to support this use case.
The big issue is that to use it properly, it has be called from an async function.
This means that roc_alloc will no longer return a pointer. Instead, it will return a Box of a Future of a pointer.

So this is pointing more and more toward roc effects being configurable.
Since roc_alloc is completely controlled by the roc runtime, roc could theoretically generate all the wrappers and contiunations of the allocating.

Another less nice option is to pass the tokio task id into roc.
And have roc feed that into roc_alloc.
Then roc_alloc, could access a global hash map of id to arena.
Since this is global instead of task local, it does not have to be async.
That being said, it will require probably require some limited locking.