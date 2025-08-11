//! Build script for FHE LLM Proxy

fn main() {
    // Simple build information
    println!(
        "cargo:rustc-env=BUILD_TIMESTAMP={}",
        chrono::Utc::now().timestamp()
    );
    println!(
        "cargo:rustc-env=BUILD_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );

    // Add security compile flags for Linux
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-arg=-Wl,-z,relro,-z,now");
    }

    // Check if we're building with GPU support
    #[cfg(feature = "gpu")]
    {
        println!("cargo:rustc-cfg=gpu_enabled");
    }

    // Set optimization flags for production builds
    #[cfg(not(debug_assertions))]
    {
        println!("cargo:rustc-env=CARGO_CFG_OPTIMIZED=1");
    }
}
