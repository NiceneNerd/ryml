fn main() {
    cxx_build::bridge("src/inner.rs")
        .file("src/shim.cc")
        .define("RYML_SINGLE_HDR_DEFINE_NOW", None)
        .define("C4CORE_SINGLE_HDR_DEFINE_NOW", None)
        .compiler("clang++")
        .compile("ryml");
    println!("cargo:rerun-if-changed=src/inner.rs");
    println!("cargo:rustc-link-lib=ryml");
}
