use bindgen;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

    // Build with make
    // TODO: use `cc` crate to build manually
    Command::new("make")
        .current_dir("minisketch/src")
        .status()
        .expect("failed to make!");

    println!("cargo:rustc-flags=-L minisketch/src -l minisketch");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
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
