hosted Effect
    exposes [Effect, after, map, always, forever, loop, method, path, fakeDBCall, Future, Request]
    imports []
    generates Effect with [after, map, always, forever, loop]

Request := Nat

# *mut dyn Future<Output = i32> is in 2 registers because it is a fat pointer with a size.
Future := [T Nat Nat]

Method : [
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
    Other,
]

method : Request -> Effect Method

path : Request -> Effect Str

fakeDBCall : U64 -> Effect Future