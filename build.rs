fn main() {
    cxx_build::bridge("src/inner.rs")
        .define("RYML_SINGLE_HDR_DEFINE_NOW", None)
        .define("C4CORE_SINGLE_HDR_DEFINE_NOW", None)
        .flag_if_supported("-std=c++17")
        .compile("ryml");
    println!("cargo:rerun-if-changed=src/inner.rs");
    println!("cargo:rerun-if-changed=src/shim.cc");
    println!("cargo:rerun-if-changed=include/shim.h");
    println!("cargo:rerun-if-changed=include/ryml.h");
    println!("cargo:rustc-link-lib=ryml");
}
