fn main() -> miette::Result<()> {
    let path = std::path::PathBuf::from("./"); // include path
    let mut b = autocxx_build::Builder::new("src/main.rs", &[&path]).build()?;
        // This assumes all your C++ bindings are in main.rs
    b.flag_if_supported("-std=c++14")
     .compile("libleap-sys-demo"); // arbitrary library name, pick anything
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rustc-link-lib=Leap");
    // Add instructions to link to any C++ libraries you need.
    Ok(())
}