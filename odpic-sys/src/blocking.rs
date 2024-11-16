//! functions which may be blocked by network-roundtrips
//!
//! When the `separate_blocking` feature is enabled, functions whose column
//! `Round-Trips?` value in [ODPI-C Function Round-Trips] is `Yes` or `Maybe`
//! are in this module.
//!
//! There are no functions when the feature is disabled.
//!
//! [ODPI-C Function Round-Trips]: https://odpi-c.readthedocs.io/en/latest/user_guide/round_trips.html

#[cfg(feature = "separate_blocking")]
use crate::*;
#[cfg(feature = "separate_blocking")]
include!("bindings_blocking.rs");
