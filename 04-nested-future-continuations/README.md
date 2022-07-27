Turns out this way of doing things is unnecessary.
Instead of passing the Future through Roc with an effect.
Just pass the args for calling the future back to the host with a continuation.
Match of the return union, call the correct future with args, and call the Roc continuation.
No need to allocate a box and pass the future around...yay!!!



This is the proof of concept for generic roc async as a whole.
If this works, we should be able to setup an async web server or do other crazy things.
Of course, this is very manual here, but that is not what I am trying to solve currently.

It is a combination of the above examples.