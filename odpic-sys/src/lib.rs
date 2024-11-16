#![doc = include_str!("../README.md")]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod blocking;
pub mod dpi_impl;

include!("bindings.rs");
#[cfg(not(feature = "separate_blocking"))]
include!("bindings_blocking.rs");
