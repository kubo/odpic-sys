use bindgen::{
    callbacks::{ItemInfo, ItemKind, ParseCallbacks},
    Builder, RustTarget,
};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
enum RoundTrip {
    Yes,
    No,
    Maybe,
}

type FuncTypeMap = HashMap<String, RoundTrip>;

// Functions not listed in round_trips.html
const ADDITIONAL_FUNCS: &[(&str, RoundTrip)] = &[
    // deprecated in ODPI-C v5.0.0
    ("dpiSodaDb_freeCollectionNames", RoundTrip::No),
];

// Return the mapping from ODPI-C function names to whether the function
// requires network round trips to the database.
// See: https://odpi-c.readthedocs.io/en/latest/user_guide/round_trips.html
fn function_types(dir: &str) -> Result<FuncTypeMap, Box<dyn Error>> {
    let mut map = HashMap::new();
    let f = File::open(format!("{}/odpi/doc/src/user_guide/round_trips.rst", dir))?;
    let reader = BufReader::new(f);
    let mut func_name = None::<String>;
    for (lineno, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if let Some(fname) = func_name {
            let round_trip = if line.ends_with(" Yes") {
                RoundTrip::Yes
            } else if line.ends_with(" No") {
                RoundTrip::No
            } else if line.ends_with(" Maybe") {
                RoundTrip::Maybe
            } else {
                panic!("unexpected format '{}' at line {}", line, lineno + 1);
            };
            map.insert(fname, round_trip);
            func_name = None;
        } else {
            match (line.find("* - :func:`"), line.find("()`")) {
                (Some(s), Some(e)) if s < e => {
                    func_name = Some(line[(s + 11)..e].to_string());
                }
                _ => (),
            }
        }
    }
    Ok(map)
}

#[derive(Clone, Debug)]
struct Callbacks(Rc<FuncTypeMap>);

impl ParseCallbacks for Callbacks {
    fn generated_name_override(&self, item_info: ItemInfo<'_>) -> Option<String> {
        if let ItemKind::Function = item_info.kind {
            if self.0.get(item_info.name).is_none() {
                println!(
                    "WARNING: {} isn't listed in round_trips.rst",
                    item_info.name
                );
            }
        }
        None
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let rust_target = RustTarget::Stable_1_59;
    let dir = format!("{}/../odpic-sys", env!("CARGO_MANIFEST_DIR"));

    let mut func_type_map = function_types(&dir)?;
    for (func_name, round_trip) in ADDITIONAL_FUNCS {
        func_type_map.insert(func_name.to_string(), *round_trip);
    }
    let func_type_map = Rc::new(func_type_map);
    let callbacks = Callbacks(func_type_map.clone());

    let mut builder = Builder::default()
        .header(format!("{}/odpi/include/dpi.h", dir))
        .allowlist_type("^dpi.*")
        .allowlist_function("^dpi.*")
        .allowlist_var("^DPI_.*")
        .bitfield_enum("dpiExecMode")
        .bitfield_enum("dpiFetchMode")
        .bitfield_enum("dpiOpCode")
        .bitfield_enum("dpiSubscrQOS")
        .prepend_enum_name(false)
        .derive_default(true)
        .parse_callbacks(Box::new(callbacks.clone()))
        .rust_target(rust_target);
    for (k, v) in func_type_map.iter() {
        if *v != RoundTrip::No {
            builder = builder.blocklist_function(k);
        }
    }
    let bindings = builder
        .generate()
        .expect("Unable to generate bindings from dpi.h");
    let f = File::create(format!("{}/src/bindings.rs", dir)).expect("Cound not open bindings.rs");
    bindings
        .write(Box::new(f))
        .expect("Couldn't write bindings!");

    let mut builder = Builder::default()
        .header(format!("{}/odpi/include/dpi.h", dir))
        .allowlist_recursively(false)
        .parse_callbacks(Box::new(callbacks.clone()))
        .rust_target(rust_target);
    for (k, v) in func_type_map.iter() {
        if *v != RoundTrip::No {
            builder = builder.allowlist_function(k);
        }
    }
    let bindings = builder
        .generate()
        .expect("Unable to generate bindings from dpi.h");
    let f = File::create(format!("{}/src/bindings_blocking.rs", dir))
        .expect("Cound not open bindings_blocking.rs");
    bindings
        .write(Box::new(f))
        .expect("Couldn't write bindings_blocking!");

    let bindings = Builder::default()
        .header(format!("{}/odpi/src/dpiImpl.h", dir))
        .allowlist_var("^DPI_.*")
        .rust_target(rust_target)
        .clang_arg(format!("-I{}/odpi/include", dir))
        .generate()
        .expect("Unable to generate bindings from dpiImpl.h");
    let f = File::create(format!("{}/src/bindings_impl.rs", dir))
        .expect("Cound not open bindings_impl.rs");
    bindings
        .write(Box::new(f))
        .expect("Couldn't write bindings_impl!");
    Ok(())
}
