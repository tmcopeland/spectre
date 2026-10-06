//! Locates and links the libsndfile C library.
//!
//! Lookup order:
//!   1. `SNDFILE_LIB_DIR` – directory containing the library (for unusual installs).
//!   2. `pkg-config sndfile`.
//!   3. Plain `-lsndfile` and let the linker search its default paths.

use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=SNDFILE_LIB_DIR");
    println!("cargo:rerun-if-changed=build.rs");

    if let Ok(dir) = env::var("SNDFILE_LIB_DIR") {
        println!("cargo:rustc-link-search=native={dir}");
        println!("cargo:rustc-link-lib=sndfile");
        return;
    }

    // We only need the library, not the headers, so ask pkg-config for libs only.
    if pkg_config::Config::new()
        .atleast_version("1.0.25")
        .probe("sndfile")
        .is_ok()
    {
        return;
    }

    println!("cargo:rustc-link-lib=sndfile");
}
