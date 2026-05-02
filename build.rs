fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
        println!("cargo:rustc-link-search=native=/usr/local/lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,/opt/homebrew/lib");
    }

    // Link libmpv — on Windows the .lib is generated from .def by CI
    println!("cargo:rustc-link-lib=mpv");
}
