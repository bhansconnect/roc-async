hosted Effect
    exposes [Effect, after, map, always, forever, loop, method, path, Request]
    imports []
    generates Effect with [after, map, always, forever, loop]

Request := Nat

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
