fn main() {
    // Add Homebrew library paths (Apple Silicon and Intel)
    println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
    println!("cargo:rustc-link-search=native=/usr/local/lib");

    // Set runtime library search path
    println!("cargo:rustc-link-arg=-Wl,-rpath,/opt/homebrew/lib");

    // Link libmpv dynamically
    println!("cargo:rustc-link-lib=mpv");
}
