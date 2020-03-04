use std::fs;
use std::process::Command;

// The idea behind this script is to help the user to have the proper environment to build and use the tool
// without requiring them to run any manual step such as installing a rustup component or remembering to run
// cargo using a `+nightly-AAAA-BB-CC` prefix. The latter is needed to make LD_LIBRARY_PATH point to the pinned
// nightly sysroot.

fn main() {
    // Make sure rustc-dev component is installed. Without it, the wrapper will fail to build.
    // We intentionally ignore the exit status as this is a best-effort attempt (rustup may not be installed).
    // Not a big fan of doing this here, especially since there's no visible output.
    let _ = Command::new("rustup").args(&["component", "add", "rustc-dev"]).status();

    // Forward the toolchain as an environment variable so that we can run the wrapper with the correct toolchain.
    let toolchain = format!("+{}", fs::read_to_string("rust-toolchain").expect("failed to read rust-toolchain file"));
    println!("cargo:rustc-env=MINVER_TOOLCHAIN={}", toolchain);

    // Rebuild this script if the toolchain file changes.
    println!("cargo:rerun-if-changed=rust-toolchain");
}
