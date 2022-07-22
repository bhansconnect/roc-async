hosted Effect
    exposes [Effect, after, map, always, forever, loop, readData]
    imports []
    generates Effect with [after, map, always, forever, loop]

# *mut dyn Future<Output = i32> is in 2 registers because it is a fat pointer with a size.
Future := [T Nat Nat]

readData : Effect Future