// odpic-sys - raw binding to ODPI-C
//
// URL: https://github.com/kubo/odpic-sys
//
//-----------------------------------------------------------------------------
// Copyright (c) 2024-2025 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

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
