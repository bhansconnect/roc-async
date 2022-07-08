So we may need the ability to return continuations to the host.
Potentially with a compiler flag that just tells roc that all Effects are async.
That being said, this theoretically can be done manually if Roc generates some of these types correclty.

Lets say we are making a webserver.
the host functions are readDB and writeDB.

Those are both async functions and instead of returning directly, they return a future.
As such roc code needs to return the future to the runtime before continuing.
The idea is to return a future union.
Future = [ ReadDB ReadFuture (ReadData -> Future), WriteDB WriteFuture (() -> Future), Complete OutputType ]

Now when we need to call a host function, we call it. and build one of these union values.
We pass a continuation in if necessary.
Finally we return that to the host so it can wait on the future.
After it is done, it will call a continuation if it exist or just complete.

simple example function

```
main = \dbKey ->
    mainData <- Task.await (readDB dbKey)
    relatedData <- Task.await (readDB mainData.relatedKey)
    newData = {mainData & someVal: mainData.someVal + relatedData.someVal}
    _ <- Task.await (writeDB dbKey newData)
    Task.suceed {}
```

So this function reads two times from the database and then writes back an updated value.
The execution would end up being:

Call main => ReadDB (readDB dbKey) (\mainData -> ...)
Run the Future in the host and get data from the database.

Call (\mainData -> ...) => ReadDB (readBD mainData.relatedKey) (\relatedData -> ...)
Run the Future in the host and get data from the database.

Call (\relatedData -> ...) => WriteDB (writeDB dbKey newData) (\_ -> ...)
Run the Future in the host and set the value in the database

Call (\_ -> ...) => Complete {}
Host continues on as it sees fit.

The futures would all be opaque to Roc and just returned back to the host.
They could be any sort of value/wrapper which should support multiple languages better.


This is actually very similar to how an async function is actually desugared in Rust.
An important distinction is that Rust does not return continuatoins.
Instead, rust builds a state machine splitting up the function.
This should be more efficient than closures but obviously takes more work to generate.

So the above function would be more similar to this:
struct AsyncFuture {
    read_db_future_one: ReadDBFuture,
    read_val_one: ValType,
    read_db_future_two: ReadDBFuture,
    read_val_two: ValType,
    write_db_future_one: WriteDBFuture,
    state: State,
}
enum State {
    AwaitingReadDBOne,
    AwaitingReadDBTwo,
    AwaitingWriteDBOne,
    Done,
}
loop {
    match self.state {
        State::AwaitingReadDBOne => match self.read_db_future_one.poll(..) {
            Poll::Ready(read_val) => {
                self.state = State::AwaitingReadDBTwo;
                self.read_val_one = read_val;
            },
            Poll::Pending => return Poll::Pending,
        }
        State::AwaitingReadDBTwo => match self.read_db_future_two.poll(..) {
            Poll::Ready(read_val) => {
                self.state = State::AwaitingWriteDBOne;
                self.read_val_two = read_val;
            },
            Poll::Pending => return Poll::Pending,
        }
        State::AwaitingWriteDBOne => match self.write_db_future_one.poll(..) {
            Poll::Ready(()) => self.state = State::Done,
            Poll::Pending => return Poll::Pending,
        }
        State::Done => return Poll::Ready(()),
    }
}