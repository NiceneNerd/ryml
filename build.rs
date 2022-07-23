fn main() {
    cxx_build::bridge("src/lib.rs")
        .define("RYML_SINGLE_HDR_DEFINE_NOW", None)
        .define("C4CORE_SINGLE_HDR_DEFINE_NOW", None)
        .compiler("clang++")
        .compile("ryml");
    println!("cargo:rustc-link-lib=ryml");
}
