use crate::bindings;

// BIG QUESTION: how do we provide Rust implementations to C++ classes?
// One option could be to use the `cpp` crate, but we should be able
// to do something better. Maybe figure out what symbols it looks for.
//
// After all, it must use symbols via a handlerton

// Okay: I think that if I understand things correctly, the `ha_x` interfaces are
// meant mostly for access by the server. Our implementation (I think) only needs
// to provide the virtual functions somehow.
//
// Maybe the way to do this is to create a vtable struct with function pointers and
// assign the first `handler` item to it

/// This is our way of faking inheritance
#[repr(C)]
struct HandlerWrapper<T: Handler> {
    handler: bindings::handler,
    this: T,
}

pub trait Handler {}
