//! This file runs `cmake` as needed, then `bindgen` to produce the rust bindings

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::process::Command;

use bindgen::callbacks::{DeriveInfo, MacroParsingBehavior, ParseCallbacks};
use bindgen::EnumVariation;

// `math.h` seems to double define some things, To avoid this, we ignore them.
const IGNORE_MACROS: [&str; 20] = [
    "FE_DIVBYZERO",
    "FE_DOWNWARD",
    "FE_INEXACT",
    "FE_INVALID",
    "FE_OVERFLOW",
    "FE_TONEAREST",
    "FE_TOWARDZERO",
    "FE_UNDERFLOW",
    "FE_UPWARD",
    "FP_INFINITE",
    "FP_INT_DOWNWARD",
    "FP_INT_TONEAREST",
    "FP_INT_TONEARESTFROMZERO",
    "FP_INT_TOWARDZERO",
    "FP_INT_UPWARD",
    "FP_NAN",
    "FP_NORMAL",
    "FP_SUBNORMAL",
    "FP_ZERO",
    "IPPORT_RESERVED",
];

const DERIVE_COPY_NAMES: [&str; 1] = ["enum_field_types"];

#[derive(Debug)]
struct BuildCallbacks(HashSet<String>);

impl ParseCallbacks for BuildCallbacks {
    /// Ignore macros that are in the ignored list
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        if self.0.contains(name) {
            MacroParsingBehavior::Ignore
        } else {
            MacroParsingBehavior::Default
        }
    }

    /// Use a converter to turn doxygen comments into rustdoc
    fn process_comment(&self, comment: &str) -> Option<String> {
        Some(doxygen_rs::transform(comment))
    }

    fn add_derives(&self, _info: &DeriveInfo<'_>) -> Vec<String> {
        if DERIVE_COPY_NAMES.contains(&_info.name) {
            vec!["Copy".to_owned()]
        } else {
            vec![]
        }
    }
}

impl BuildCallbacks {
    fn new() -> Self {
        Self(IGNORE_MACROS.into_iter().map(|s| s.to_owned()).collect())
    }
}

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/wrapper.h");
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cmake_path = cargo_dir.parent().unwrap().parent().unwrap();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let profile = std::env::var("PROFILE").unwrap();

    let cmake_build_flag = match profile.as_str() {
        "release" => "-DBUILD_CONFIG=mysql_release",
        "debug" => "-DCMAKE_BUILD_TYPE=Debug",
        _ => panic!(),
    };

    // Run cmake to configure only
    Command::new("cmake")
        .arg(cmake_path)
        .arg("-B")
        // .arg("../../")
        .arg(&out_dir)
        .arg(cmake_build_flag)
        .output()
        .expect("failed to invoke cmake");

    // panic!("OD is {out_dir}");
    // let dst = cmake::build("../../");
    // let dst = cmake::Config::new("../../")
    //     .build();
    // println!("cargo:rustc-link-search=native={}", dst.display());
    // println!("cargo:rustc-link-lib=static=foo");

    // The bindgen::Builder is the main entry point to bindgen, and lets you
    // build up options for the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/wrapper.h")
        .clang_arg(format!("-I{}", cmake_path.join("include").display()))
        .clang_arg(format!("-I{}", cmake_path.join("sql").display()))
        .clang_arg(format!("-I{}", out_dir.join("include").display()))
        .clang_arg(format!("-I{}", out_dir.join("sql").display()))
        .clang_arg("-xc++")
        .clang_arg("-std=c++17")
        // Don't derive copy for structs
        .derive_copy(false)
        // Use rust-style enums labeled with non_exhaustive to represent C enums
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: true,
        })
        // Fix math.h double defines
        .parse_callbacks(Box::new(BuildCallbacks::new()))
        // LLVM has some issues with long dobule and ABI compatibility
        // disabling the only relevant function here to suppress errors
        .blocklist_function("strfroml")
        .blocklist_function("strfromf64x")
        .blocklist_function("strtof64x_l")
        .blocklist_function("strtof64x")
        .blocklist_function("strtold")
        .blocklist_function("strtold_l")
        // qvct, evct, qfcvt_r, ...
        .blocklist_function("[a-z]{1,2}cvt(?:_r)?")
        // c++ things that aren't supported
        .blocklist_item("List_iterator")
        .blocklist_type("std::char_traits")
        .opaque_type("std_.*")
        .blocklist_item("std_basic_string")
        .blocklist_item("std_collate.*")
        .blocklist_item("__gnu_cxx.*")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("couldn't write bindings");
}
