use std::process::Command;

use anyhow::Result;
use serde::Serialize;

use crate::{
    cli::{GlobalOptions, NetPingArgs},
    error, output,
};

#[derive(Debug, Serialize)]
struct PingOutput {
    host: String,
    success: bool,
    status_code: Option<i32>,
}

pub fn run(args: &NetPingArgs, global: &GlobalOptions) -> Result<()> {
    if args.count == 0 {
        return Err(error::msg("ping count must be greater than zero"));
    }
    let mut command = Command::new("ping");
    configure_ping(&mut command, &args.host, args.count, args.timeout);
    let result = command
        .output()
        .map_err(|err| error::msg(format!("failed to execute OS ping command: {err}")))?;
    let success = result.status.success();
    let output_value = PingOutput {
        host: args.host.clone(),
        success,
        status_code: result.status.code(),
    };
    output::write_or_json(
        global,
        || {
            println!(
                "PING {}: {}",
                args.host,
                if success { "success" } else { "failed" }
            );
            if !global.quiet {
                let stdout = String::from_utf8_lossy(&result.stdout);
                for line in stdout.lines().take(12) {
                    println!("{line}");
                }
                let stderr = String::from_utf8_lossy(&result.stderr);
                for line in stderr.lines().take(12) {
                    eprintln!("{line}");
                }
            }
            Ok(())
        },
        &output_value,
    )?;
    if success {
        Ok(())
    } else {
        Err(error::silent_exit(1))
    }
}

#[cfg(windows)]
fn configure_ping(command: &mut Command, host: &str, count: u16, timeout_ms: u64) {
    command.args([
        "-n",
        &count.to_string(),
        "-w",
        &timeout_ms.to_string(),
        host,
    ]);
}

#[cfg(target_os = "macos")]
fn configure_ping(command: &mut Command, host: &str, count: u16, timeout_ms: u64) {
    command.args([
        "-c",
        &count.to_string(),
        "-W",
        &timeout_ms.to_string(),
        host,
    ]);
}

#[cfg(all(unix, not(target_os = "macos")))]
fn configure_ping(command: &mut Command, host: &str, count: u16, timeout_ms: u64) {
    let seconds = ((timeout_ms + 999) / 1000).max(1);
    command.args(["-c", &count.to_string(), "-W", &seconds.to_string(), host]);
}
