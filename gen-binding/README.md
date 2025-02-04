# gen-binding

The program reads `odpi/include/dpi.h` and `odpi/doc/src/user_guide/round_trips.rst`
and creates `odpic-sys/src/bindings.rs` and `odpic-sys/src/bindings_blocking.rs`.

It also reads `odpi/src/dpiImpl.h` and creates `odpic-sys/src/bindings_impl.rs`.
