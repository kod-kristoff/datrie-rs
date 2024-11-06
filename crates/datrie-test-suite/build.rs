fn main() {
    println!("cargo:rustc-link-search=native=target/debug");
    println!("cargo:rustc-link-lib=dylib=datrie");
}
