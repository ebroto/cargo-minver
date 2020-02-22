use std::env;
use std::process::Command;
use std::str;

use anyhow::{Context, Result};

use cargo_minver::{ipc, wrapper, SERVER_PORT_ENV};

fn main() -> Result<()> {
    let mut args = env::args().collect::<Vec<_>>();
    // Remove "rustc" from the argument list
    args.remove(1);

    if args.iter().any(|arg| arg == "--print=cfg") {
        // Cargo is collecting information about the crate: passthrough to the actual compiler.
        Command::new("rustc")
            .args(&args[1..])
            .status()
            .context("failed to execute rustc")?;
        Ok(())
    } else {
        // Cargo is building a crate: run the compiler using our wrapper.
        args.extend(vec![
            "--sysroot".to_string(),
            fetch_sysroot().context("could not fetch sysroot")?,
        ]);
        let analysis = wrapper::run_compiler(&args)?;

        // Send the results to the server.
        let port = server_port_from_env().context("invalid server port in environment")?;
        ipc::send_message(port, &ipc::Message::Analysis(analysis))
            .context("failed to send analysis result to server")?;
        Ok(())
    }
}

// TODO: full-fledged sysroot detection (see e.g. clippy)
fn fetch_sysroot() -> Result<String> {
    let output = Command::new("rustc").args(vec!["--print", "sysroot"]).output()?;
    let sysroot = str::from_utf8(&output.stdout)?;
    Ok(sysroot.trim_end().to_string())
}

fn server_port_from_env() -> Result<u16> {
    let port_var = env::var(SERVER_PORT_ENV)?;
    let port = port_var.parse()?;
    Ok(port)
}
