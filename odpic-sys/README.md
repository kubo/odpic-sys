# odpic-sys

The crate provides low-level Rust bindings to [ODPI-C].

## Usage

Put this in your `Cargo.toml`:

```text
[dependencies]
odpic-sys = "0.1.1"
```

The following Cargo features are supported:

* `separate_blocking` separates functions which may be blocked by network round-trips.

  When the feature is enabled, functions whose column `Round-Trips?` value in
  [ODPI-C Function Round-Trips] is `Yes` or `Maybe` are moved from the top-level
  module to the [`blocking`] module.

## Note about doc comments

Doc comments in this crate are verbatim copies of ODPI-C doc.
They are written for C language and may be inappropriate for Rust.

## Compile-time Requirements

See [`Compile-time Requirements`](https://docs.rs/cc/latest/cc/#compile-time-requirements).

## Relation between odpic-sys version and ODPI-C version

| odpic-sys version | [ODPI-C version] | [RustTarget] | note |
|-------|-------|------|---|
| 0.2.0 |   -   |   -  | not yet published |
| 0.1.1 | 5.4.1 | 1.59 |   |
| 0.1.0 | 5.4.0 | 1.59 |   |

## License

Same with ODPI-C

1. [the Universal Permissive License v 1.0 or at your option, any later version](http://oss.oracle.com/licenses/upl); and/or
2. [the Apache License v 2.0](http://www.apache.org/licenses/LICENSE-2.0).

## Copyrights

Copyrights of `src/binding_*.rs` files belong to respective owners.
The files were generated from [ODPI-C] header files and documents.

[`dpi.h`]: https://github.com/oracle/odpi/blob/main/include/dpi.h
[ODPI-C]: https://oracle.github.io/odpi/
[ODPI-C Function Round-Trips]: https://odpi-c.readthedocs.io/en/latest/user_guide/round_trips.html
[ODPI-C version]: https://odpi-c.readthedocs.io/en/latest/releasenotes.html
[RustTarget]: https://docs.rs/bindgen/0.70.1/bindgen/enum.RustTarget.html
