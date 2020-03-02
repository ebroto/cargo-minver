use std::fs;
use std::process::Command;

// The idea behind this script is to help the user to have the proper environment to build and use the tool,
// without requiring them to run any manual step such as installing a rustup component or remembering to run
// cargo using a `+nightly-AAAA-BB-CC` prefix.

fn main() {
    // Make sure rustc-dev component is installed. Without it, the wrapper will fail to build.
    // Not a big fan of doing this here, especially since there's no visible output.
    Command::new("rustup")
        .args(&["component", "add", "rustc-dev"])
        .status()
        .expect("failed to add rustc-dev rustup component");

    // Forward the toolchain as an environment variable so that we can run the wrapper with the correct toolchain.
    let toolchain = format!("+{}", fs::read_to_string("rust-toolchain").expect("failed to read rust-toolchain file"));
    println!("cargo:rustc-env=MINVER_TOOLCHAIN={}", toolchain);

    // Rebuild this script if the toolchain file changes.
    println!("cargo:rerun-if-changed=rust-toolchain");
}
