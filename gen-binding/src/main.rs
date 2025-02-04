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

use anyhow::{bail, Result};
use bindgen::{
    callbacks::{IntKind, ItemInfo, ItemKind, ParseCallbacks},
    Builder, RustTarget,
};
use odpic_sys::doc::{OdpicDoc, RoundTrips, UnderlyingType, DataKind};
use regex::{Match, Regex};
use std::borrow::Cow;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::str;

#[derive(Clone, Debug)]
struct Callbacks(Rc<OdpicDoc>);

impl ParseCallbacks for Callbacks {
    fn generated_name_override(&self, item_info: ItemInfo<'_>) -> Option<String> {
        if let ItemKind::Function = item_info.kind {
            if self.0.round_trips_map.get(item_info.name).is_none() {
                println!(
                    "WARNING: {} isn't listed in round_trips.rst",
                    item_info.name
                );
            }
        }
        None
    }

    fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
        if name == "DPI_SUCCESS" {
            return Some(IntKind::I32);
        }
        Some(match self.0.find_underlying_type(name)? {
            UnderlyingType::Uint8 => IntKind::U8,
            UnderlyingType::Uint16 => IntKind::U16,
            UnderlyingType::Uint32 => IntKind::U32,
        })
    }
}

struct DocComment {
    re: Regex,
    doc: Rc<OdpicDoc>,
    struct_or_union_name: String,
}

impl DocComment {
    fn new() -> Result<DocComment> {
        let mut doc = OdpicDoc::read_yaml()?;
        for dt in &doc.data_types {
            if dt.kind == DataKind::Enum {
                let desc = doc.name_to_desc.get_mut(&dt.name).unwrap();
                desc.push_str("\n");
                desc.push_str("Value | Description\n");
                desc.push_str("---|---\n");
                for m in &dt.members {
                    desc.push_str("`");
                    desc.push_str(&m.name);
                    desc.push_str("` | ");
                    desc.push_str(&m.desc.replace('\n', " "));
                    desc.push_str("\n");
                }
                for m in &dt.members {
                    let desc = doc.name_to_desc.get_mut(&m.name).unwrap();
                    *desc = format!("See [`{}`]", dt.name);
                }
            }
        }
        Ok(DocComment {
            re: Regex::new(r"^(\s*)pub (?:(const|fn|struct|union|type) (\w+)|(\w+):)")?,
            doc: Rc::new(doc),
            struct_or_union_name: String::new(),
        })
    }

    fn find_desc<'a, 'b>(&'a mut self, line: &'b str) -> Result<(&'a str, &'b str)> {
        let not_found = Ok(("", ""));
        let caps = if let Some(caps) = self.re.captures(line) {
            caps
        } else {
            return not_found;
        };
        let name: Cow<str> = match (
            caps.get(3).as_ref().map(Match::as_str),
            caps.get(4).as_ref().map(Match::as_str),
        ) {
            (Some(name), None) => {
                self.struct_or_union_name = match caps.get(2).as_ref().map(Match::as_str) {
                    Some("struct") | Some("union") => name.to_string(),
                    _ => "".into(),
                };
                Cow::Borrowed(name)
            }
            (None, Some(name)) => format!("{}::{}", self.struct_or_union_name, name).into(),
            (_, _) => bail!("Unexpected captures for line {}", line),
        };
        if let Some(desc) = self.doc.find_desc(name.as_ref()) {
            Ok((desc, caps.get(1).unwrap().as_str()))
        } else {
            println!("WARNING: {} has no description.", name);
            not_found
        }
    }
}

fn main() -> Result<()> {
    let rust_target = RustTarget::Stable_1_59;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dir = manifest_dir.to_owned() + "/../odpic-sys";
    let dpi_h = dir.clone() + "/odpi/include/dpi.h";
    let dpi_impl_h = dir.clone() + "/odpi/src/dpiImpl.h";
    let mut dc = DocComment::new()?;
    let callbacks = Callbacks(dc.doc.clone());

    // additional types found in doc but not in dpi.h
    let contents = "#include <stdint.h>\n\
                    typedef uint32_t dpiJsonOptions;\n\
                    typedef uint32_t dpiSodaFlags;\n";

    let mut builder = Builder::default()
        .header(&dpi_h)
        .header_contents("dpi_additional.h", contents)
        .allowlist_type("^dpi.*")
        .allowlist_function("^dpi.*")
        .allowlist_var("^DPI_.*")
        .prepend_enum_name(false)
        .derive_default(true)
        .parse_callbacks(Box::new(callbacks.clone()))
        .rust_target(rust_target);
    for (k, v) in dc.doc.round_trips_map.iter() {
        if *v != RoundTrips::No {
            builder = builder.blocklist_function(k);
        }
    }
    write_bindings(builder, &mut dc, format!("{}/src/bindings.rs", dir))?;

    let mut builder = Builder::default()
        .header(&dpi_h)
        .allowlist_recursively(false)
        .parse_callbacks(Box::new(callbacks.clone()))
        .rust_target(rust_target);
    for (k, v) in dc.doc.round_trips_map.iter() {
        if *v != RoundTrips::No {
            builder = builder.allowlist_function(k);
        }
    }
    write_bindings(
        builder,
        &mut dc,
        format!("{}/src/bindings_blocking.rs", dir),
    )?;

    let bindings = Builder::default()
        .header(&dpi_impl_h)
        .allowlist_var("^DPI_.*")
        .rust_target(rust_target)
        .clang_arg(format!("-I{}/odpi/include", dir))
        .generate()?;
    bindings.write_to_file(format!("{}/src/bindings_impl.rs", dir))?;
    Ok(())
}

fn write_bindings<P: AsRef<Path>>(builder: Builder, dc: &mut DocComment, path: P) -> Result<()> {
    let mut buf = Vec::new();
    builder.generate()?.write(Box::new(&mut buf))?;
    let mut f = File::create(path)?;
    for line in str::from_utf8(&buf)?.lines() {
        let (desc, spaces) = dc.find_desc(line)?;
        if !desc.is_empty() {
            writeln!(f, "")?;
            for desc_line in desc.lines() {
                writeln!(f, "{}/// {}", spaces, desc_line)?;
            }
        }
        writeln!(f, "{}", line)?;
    }
    Ok(())
}
