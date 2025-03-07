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

//! This module provides constants in [`dpiImpl.h`].
//!
//! Constants here don't follow semantic versioning because of non-public ones.
//!
//! [`dpiImpl.h`]: https://github.com/oracle/odpi/blob/main/src/dpiImpl.h

include!("bindings_impl.rs");
