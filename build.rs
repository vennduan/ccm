use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Check for SQLCipher
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "windows" {
        // On Windows, we use bundled SQLite with SQLCipher
        println!("cargo:rustc-link-lib=sqlite3");
    } else if target_os == "macos" {
        // On macOS, check for SQLCipher via Homebrew
        println!("cargo:rustc-link-search=/usr/local/opt/sqlcipher/lib");
        println!("cargo:rustc-link-lib=sqlcipher");
    } else if target_os == "linux" {
        // On Linux, try to find SQLCipher in common locations
        println!("cargo:rustc-link-lib=sqlcipher");
    }
}
