use std::env;

fn main() {
    // Windows SQLCipher with pre-built static libraries
    // Libraries are in project root: lib/ and include/
    // See docs/vcpkg-openssl-static-linking.md for build instructions

    #[cfg(all(windows, target_env = "msvc"))]
    {
        // Get project root directory
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        // Link OpenSSL libraries (order matters - libssl depends on libcrypto)
        println!("cargo:rustc-link-lib=static=libcrypto");
        println!("cargo:rustc-link-lib=static=libssl");

        // Windows system libraries that OpenSSL depends on
        println!("cargo:rustc-link-lib=ws2_32");
        println!("cargo:rustc-link-lib=gdi32");
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=crypt32");
        println!("cargo:rustc-link-lib=user32");

        // Add library search path (project root/lib/)
        println!("cargo:rustc-link-search={}/lib", manifest_dir);

        // Also search project root for sqlite3_static.lib
        println!("cargo:rustc-link-search={}", manifest_dir);

        // For debugging: print the paths being used
        println!("cargo:warning=SQLCipher lib dir: {}/lib", manifest_dir);
    }
}
