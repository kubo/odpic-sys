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

use serde::Deserialize;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::result;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    YamlError(serde_yaml::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> result::Result<(), fmt::Error> {
        match self {
            Error::IoError(err) => err.fmt(f),
            Error::YamlError(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Error {
        Error::YamlError(err)
    }
}

type Result<T> = result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum Mode {
    #[serde(rename = "IN")]
    In,
    #[serde(rename = "OUT")]
    Out,
    #[serde(rename = "IN/OUT")]
    InOut,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MemberInfo {
    pub name: String,
    pub desc: String,
    pub c_type: Option<String>,
    pub mode: Option<Mode>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum UnderlyingType {
    #[serde(rename = "uint8_t")]
    Uint8,
    #[serde(rename = "uint16_t")]
    Uint16,
    #[serde(rename = "uint32_t")]
    Uint32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum DataKind {
    #[serde(rename = "enum")]
    Enum,
    #[serde(rename = "opaque struct")]
    OpaqueStruct,
    #[serde(rename = "struct")]
    Struct,
    #[serde(rename = "union")]
    Union,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum RoundTrips {
    Yes,
    No,
    Maybe,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub desc: String,
    pub round_trips: RoundTrips,
    #[serde(rename = "return")]
    pub rettype: String,
    pub params: Vec<MemberInfo>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DataTypeInfo {
    pub name: String,
    pub kind: DataKind,
    pub desc: String,
    pub underlying_type: Option<UnderlyingType>,
    #[serde(default)]
    pub members: Vec<MemberInfo>,
    #[serde(default)]
    pub functions: Vec<FunctionInfo>,
}

impl DataTypeInfo {
    pub fn read_yaml() -> Result<Vec<DataTypeInfo>> {
        let f = File::open(format!("{}/doc.yaml", env!("CARGO_MANIFEST_DIR")))?;
        Ok(serde_yaml::from_reader(f)?)
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct OdpicDoc {
    pub data_types: Vec<DataTypeInfo>,
    pub round_trips_map: HashMap<String, RoundTrips>,
    pub underlying_type_map: HashMap<String, UnderlyingType>,
    pub name_to_desc: HashMap<String, String>,
}

impl OdpicDoc {
    pub fn read_yaml() -> Result<OdpicDoc> {
        let data_types: Vec<DataTypeInfo> = DataTypeInfo::read_yaml()?;
        let mut round_trips_map = HashMap::new();
        for dt in &data_types {
            for func in &dt.functions {
                round_trips_map.insert(func.name.clone(), func.round_trips);
            }
        }
        let mut underlying_type_map = HashMap::new();
        for dt in &data_types {
            if let Some(underlying_type) = dt.underlying_type {
                for m in &dt.members {
                    underlying_type_map.insert(m.name.clone(), underlying_type);
                }
            }
        }
        let mut name_to_desc = HashMap::new();
        for dt in &data_types {
            name_to_desc.insert(dt.name.clone(), dt.desc.clone());
            match dt.kind {
                DataKind::Enum => {
                    for m in &dt.members {
                        name_to_desc.insert(m.name.clone(), m.desc.clone());
                    }
                }
                DataKind::Struct | DataKind::Union => {
                    for m in &dt.members {
                        name_to_desc.insert(format!("{}::{}", dt.name, m.name), m.desc.clone());
                    }
                }
                DataKind::OpaqueStruct => (),
            }
            for f in &dt.functions {
                name_to_desc.insert(f.name.clone(), f.desc.clone());
            }
        }
        Ok(OdpicDoc {
            data_types,
            round_trips_map,
            underlying_type_map,
            name_to_desc,
        })
    }

    pub fn find_desc<Q>(&self, name: &Q) -> Option<&str>
    where
        String: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.name_to_desc.get(name).map(String::as_ref)
    }

    pub fn find_underlying_type<Q>(&self, name: &Q) -> Option<UnderlyingType>
    where
        String: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.underlying_type_map.get(name).copied()
    }
}
