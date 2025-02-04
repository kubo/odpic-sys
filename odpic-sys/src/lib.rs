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

#![doc = include_str!("../README.md")]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod blocking;
pub mod dpi_impl;

include!("bindings.rs");
#[cfg(not(feature = "separate_blocking"))]
include!("bindings_blocking.rs");

#[cfg(feature = "doc")]
pub mod doc;
