// Build script to handle cross-compilation issues
// Particularly for mimalloc on GNU targets

use std::env;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=TARGET");
    
    // Disable mimalloc for problematic targets
    let disable_mimalloc = target.contains("gnu") 
        || target_env == "musl"
        || target_arch == "arm"
        || target_arch == "armv7"
        || target.contains("i686")  // 32-bit targets often have issues
        || target.contains("android")
        || target.contains("freebsd");

    if disable_mimalloc {
        println!("cargo:rustc-cfg=disable_mimalloc");
        println!("cargo:warning=Disabling mimalloc for target: {}", target);
    } else {
        println!("cargo:warning=Using mimalloc for target: {}", target);
    }

    // Set additional flags for specific targets
    match target_os.as_str() {
        "windows" => {
            if target_env == "gnu" {
                println!("cargo:rustc-link-arg=-static-libgcc");
                println!("cargo:rustc-link-arg=-static-libstdc++");
            }
        }
        "linux" => {
            if target_env == "musl" {
                println!("cargo:rustc-link-arg=-static");
            }
        }
        _ => {}
    }

    // Print target information for debugging
    println!("cargo:warning=Building for target: {}", target);
    println!("cargo:warning=Target OS: {}", target_os);
    println!("cargo:warning=Target ENV: {}", target_env);
    println!("cargo:warning=Target ARCH: {}", target_arch);
}
