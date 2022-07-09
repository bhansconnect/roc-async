hosted Effect
    exposes [Effect, after, map, always, forever, loop, readData]
    imports []
    generates Effect with [after, map, always, forever, loop]

# dyn Future<Output = i32> is in 2 registers for some reason.
# probably due to dyn it passes a pointer and size?
Future := U128

readData : Effect Future