fn main() {
    const KIND: &str = "dylib";

    let libs = [
        "kernel32", //Rust includes it by default already, but just in case link anyway
        "gdi32",
        "shell32",
        "user32",
    ];

    for lib in libs {
        println!("cargo:rustc-link-lib={KIND}={lib}");
    }
}
