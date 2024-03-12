//! This file runs `cmake` as needed, then `bindgen` to produce the rust
//! bindings
//!
//! Since we want to avoid configuring if possible, we try a few things in
//! order:
//!
//! - Check if cmake args are present, if so use that built output
//! - Check if the source directory root can be used, use that if so
//! - Configure it outselves, output in a temp directory

use std::env;
use std::path::PathBuf;
use std::process::Command;

use bindgen::callbacks::{DeriveInfo, ParseCallbacks};
use bindgen::{Bindings, EnumVariation};
use regex::Regex;

const DERIVE_COPY_NAMES: [&str; 1] = ["enum_field_types"];

type Error = Box<dyn std::error::Error>;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/wrapper.h");

    make_bindings();
}

fn make_bindings() {
    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // We try each of these methods to generate paths, closures so we can call
    // them lazily (since they may have side effects).
    let try_make_incl_paths = [
        include_paths_from_cmake,
        || Some(vec![mariadb_root()]),
        || Some(configure_returning_incl_paths()),
    ];

    let mut last_error = None;
    let mut success = false;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct LoggedError {
        location: &'static str,
        e: Error,
        loop_count: usize,
        paths: Vec<PathBuf>,
    }

    for (loop_count, make_pathset) in try_make_incl_paths.iter().enumerate() {
        let Some(paths) = make_pathset() else {
            continue;
        };

        // For source and output directories, add the include paths for `sql/`,
        // `include/`, and `rust/bridge/`
        let include_paths: Vec<_> = paths
            .iter()
            .flat_map(|path| {
                [
                    path.join("sql"),
                    path.join("include"),
                    path.join("rust").join("bridge"),
                ]
            })
            .collect();

        let bindings = match make_bindings_with_includes(&include_paths) {
            Ok(v) => v,
            Err(e) => {
                let le = LoggedError {
                    location: "bindgen",
                    e,
                    loop_count,
                    paths,
                };
                last_error = Some(le);
                continue; // just move to the next source paths if we fail here
            }
        };

        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("couldn't write bindings");

        success = true;
        break;
    }

    if !success {
        panic!("failed to generate bindings. errors: {last_error:#?}");
    }
}

/// Get the root of our mariadb project
fn mariadb_root() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

/// Find paths provided by CMake environment variables
fn include_paths_from_cmake() -> Option<Vec<PathBuf>> {
    if let Ok(src_dir) = env::var("CMAKE_SOURCE_DIR") {
        let Ok(dst_dir) = env::var("CMAKE_BINARY_DIR") else {
            panic!("CMAKE_SOURCE_DIR set but CMAKE_BINARY_DIR unset");
        };
        eprintln!("using paths from cmake");

        dbg!(Some(vec![PathBuf::from(src_dir), PathBuf::from(dst_dir)]))
    } else {
        eprintln!("cmake environment not set, skipping");
        None
    }
}

/// Run cmake in our temp directory
fn configure_returning_incl_paths() -> Vec<PathBuf> {
    eprintln!("no preconfigured source found, running cmake configure");

    let root = mariadb_root();
    let output_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("cmake");

    // Run cmake to configure only
    Command::new("cmake")
        .arg(format!("-S{}", root.display()))
        .arg(format!("-B{}", output_dir.display()))
        .output()
        .expect("failed to invoke cmake");

    vec![output_dir, root]
}

/// Given some include directories, see if bindgen works
fn make_bindings_with_includes(include_paths: &[PathBuf]) -> Result<Bindings, Error> {
    let incl_args: Vec<_> = include_paths
        .iter()
        .map(|path| format!("-I{}", path.display()))
        .collect();

    bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/wrapper.h")
        .header("src/handler_helper.h")
        // Fix math.h double defines
        .parse_callbacks(Box::new(BuildCallbacks))
        .parse_callbacks(Box::new(
            bindgen::CargoCallbacks::new().rerun_on_header_files(true),
        ))
        .clang_args(incl_args)
        .clang_arg("-xc++")
        .clang_arg("-std=c++17")
        // Don't derive copy for structs
        .derive_copy(false)
        // Use `core::ffi` instead of `std::os::raw`
        .use_core()
        // Will be required in a future version of `rustc`
        .wrap_unsafe_ops(true)
        // Use rust-style enums labeled with non_exhaustive to represent C enums
        .default_enum_style(EnumVariation::ModuleConsts)
        // The cpp classes need vtables
        .vtable_generation(true)
        // We allow only specific types to avoid generating too many unneeded bindings
        //
        // Bindings for dbug instrumentation
        .allowlist_item("_db_.*")
        // Bindings for all plugins
        .allowlist_item(".*PLUGIN.*")
        .allowlist_item(".*INTERFACE_VERSION.*")
        .allowlist_item("st_(maria|mysql)_plugin")
        // Items for variables
        .allowlist_item("mysql_var_.*")
        .allowlist_item("TYPELIB")
        // Items for for encryption plugins
        .allowlist_var("MY_AES.*")
        .allowlist_var("ENCRYPTION_.*.*")
        .allowlist_item("PLUGIN_.*")
        .allowlist_type("st_mariadb_encryption")
        // Items for ft parsers
        .allowlist_item("enum_ftparser_mode")
        .allowlist_item("enum_field_types")
        // Items for storage engines
        .allowlist_item("handlerton")
        .allowlist_item("handler")
        .allowlist_item(".*(ha|handler)_bridge.*")
        .allowlist_item("st_mysql_storage.*")
        .allowlist_type("TABLE(_SHARE)?")
        .allowlist_type("MYSQL_HANDLERTON.*")
        .allowlist_var("HA_.*")
        .allowlist_var("IO_SIZE")
        .allowlist_var("MAX_REF_PARTS")
        .allowlist_var("MAX_DATA_LENGTH_FOR_KEY")
        // Items for the SQL service. Note that `sql_service` (from `st_service_ref`) needs to
        // be handwritten because it doesn't seem to import with the expected values (a static vs.
        // dynamic thing).
        .allowlist_item("MYSQL_.*")
        .allowlist_type("sql_service_st")
        // Finish the builder and generate the bindings.
        .generate()
        .map_err(Into::into)
}

// /// Tell cargo how to find needed libraries
// fn configure_linkage() {
//     // Set up link libraries and directories as provided by cmake
//     if let Ok(cmake_link_libs) = env::var("CMAKE_LINK_LIBRARIES") {
//         eprintln!("link libs: {cmake_link_libs}");
//         for lib in cmake_link_libs.split(';') {
//             // Remove the extension
//             let libname = lib.split_once('.').map_or(lib, |x| x.0);
//             println!("cargo:rustc-link-lib=static={libname}");
//         }
//     }

//     if let Ok(cmake_link_dirs) = env::var("CMAKE_LINK_DIRECTORIES") {
//         eprintln!("link dirs: {cmake_link_dirs}");
//         for dir in cmake_link_dirs.split(';') {
//             println!("cargo:rustc-link-search=native={dir}");
//         }
//     }
// }

#[derive(Debug)]
struct BuildCallbacks;

impl ParseCallbacks for BuildCallbacks {
    /// Simple converter to turn doxygen comments into rustdoc
    fn process_comment(&self, comment: &str) -> Option<String> {
        let brief_re = Regex::new(r"[\\@]brief ?(.*)").unwrap();
        let param_re = Regex::new(r"[\\@]param(\[(\S+)\])? (\S+)").unwrap();
        let retval_re = Regex::new(r"[\\@]retval (\S+)").unwrap();
        let brackets_re = Regex::new(r"\[(.*)\]").unwrap();
        let doxy_pos_re = Regex::new(r"^< ?(.*)").unwrap();
        let url_re = Regex::new(
            r"(?x)
            https?://                            # scheme
            ([-a-zA-Z0-9@:%._\+~\#=]{2,256}\.)+  # subdomain
            [a-zA-Z]{2,63}                       # TLD
            \b([-a-zA-Z0-9@:%_\+.~\#?&/=]*)      # query parameters
        ",
        )
        .unwrap();

        // Add `<...>` brackets to URLs
        let comment = url_re.replace_all(comment, "<$0>");
        let comment = brief_re.replace_all(&comment, "$1\n");
        let comment = param_re.replace_all(&comment, "\n* `$3` ($2)");
        let comment = retval_re.replace_all(&comment, "\n**Returns**: $1");
        let comment = brackets_re.replace_all(&comment, r"\[$1\]");
        let comment = doxy_pos_re.replace_all(&comment, "$1");

        Some(comment.to_string())
    }

    fn add_derives(&self, _info: &DeriveInfo<'_>) -> Vec<String> {
        if DERIVE_COPY_NAMES.contains(&_info.name) {
            vec!["Copy".to_owned()]
        } else {
            vec![]
        }
    }

    fn int_macro(&self, name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        let signed_vals = [
            "MARIA_PLUGIN_INTERFACE_VERSION",
            "MYSQL_HANDLERTON_INTERFACE_VERSION",
        ];

        if signed_vals.contains(&name) {
            Some(bindgen::callbacks::IntKind::Int)
        } else {
            None
        }
    }
}
