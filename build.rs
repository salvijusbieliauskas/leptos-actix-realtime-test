fn main() {
    // Check if we're building the main binary and not just checking
    let is_build =
        std::env::var("CARGO_FEATURE_SSR").is_err() && !std::env::args().any(|arg| arg == "check");

    if is_build {
        // If we're actually building and SSR isn't enabled, print a message
        println!(
            "cargo:warning=Building without 'ssr' feature - some functionality may be disabled"
        );
    }
}
