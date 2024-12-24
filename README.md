# odpic-sys

The procject provides low-level Rust bindings to [ODPI-C].

The [odpic-sys](odpic-sys) subdirectory contains rust code for the [`odpic-sys`] crate.

The [codegen](codegen) subdirectory contains rust code to generate bindings files in [odpic-sys/src](odpic-sys/src).

## Steps to update odpyc-sys to follow ODPI-C

1. Update ODPI-C and bindings files

   ```shell
   cd odpic-sys/odpi
   git fetch
   git checkout ODPIC-TAG-NAME
   cd ../../codegen
   cargo run
   ```

2. Update `odpic-sys/Cargo.toml` and `odpic-sys/README.md`

[ODPI-C]: https://oracle.github.io/odpi/
[`odpic-sys`]: https://docs.rs/odpic-sys
