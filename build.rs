use bindgen;
use cc;
use std::env;
use std::fs;
use std::fs::read_dir;
use std::path::PathBuf;

fn fail_on_empty_directory(name: &str) {
    if fs::read_dir(name).unwrap().count() == 0 {
        println!(
            "The `{}` directory is empty, did you forget to pull the submodules?",
            name
        );
        println!("Try `git submodule update --init --recursive`");
        panic!();
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=minisketch/");

    fail_on_empty_directory("minisketch");

    build_lib();
    generate_bindings();

    println!("cargo:rustc-flags=-L minisketch/src -l minisketch");
}

fn build_lib() {
    // Collect minisketch.cpp and .cpp files from fields/ directory
    let fields = read_dir("minisketch/src/fields").unwrap();
    let src_files = read_dir("minisketch/src")
        .unwrap()
        .chain(fields)
        .map(|f| f.unwrap())
        .filter(|f| !f.file_name().to_string_lossy().contains("test-exhaust.cpp"))
        .filter(|f| !f.file_name().to_string_lossy().contains("bench.cpp"))
        .filter(|f| f.file_name().to_string_lossy().ends_with(".cpp"))
        .map(|f| f.path())
        .collect::<Vec<_>>();

    // Build minisketch library
    cc::Build::new()
        .files(src_files)
        //.include("minisketch/src")
        .cpp(true)
        .opt_level(2)
        .debug(false)
        .warnings(false)
        .extra_warnings(false)
        .flag("-mpclmul")
        .flag("-g0")
        .flag("-std=c++11")
        .define("HAVE_CLZ", None)
        .compile("libminisketch.a")
}

fn generate_bindings() {
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("minisketch/include/minisketch.h")
        // Whitelist minisketch type
        .whitelist_type("minisketch")
        .opaque_type("minisketch")
        // We'll redefine Copy and Clone by utilizing minisketch's minisketch_clone() and minisketch_destroy()
        .no_copy("minisketch")
        // Whitelist minisketch library functions
        .whitelist_function("minisketch_create")
        .whitelist_function("minisketch_bits")
        .whitelist_function("minisketch_capacity")
        .whitelist_function("minisketch_implementation")
        .whitelist_function("minisketch_set_seed")
        .whitelist_function("minisketch_clone")
        .whitelist_function("minisketch_destroy")
        .whitelist_function("minisketch_serialized_size")
        .whitelist_function("minisketch_serialize")
        .whitelist_function("minisketch_deserialize")
        .whitelist_function("minisketch_add_uint64")
        .whitelist_function("minisketch_merge")
        .whitelist_function("minisketch_decode")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
