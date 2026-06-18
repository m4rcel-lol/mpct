use std::{
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::Result;
use serde::Serialize;

use crate::{
    cli::{GlobalOptions, NetPortArgs, NetPortscanArgs},
    error, output,
};

#[derive(Debug, Serialize)]
struct PortStatus {
    host: String,
    port: u16,
    open: bool,
}

pub fn run_port(args: &NetPortArgs, global: &GlobalOptions) -> Result<()> {
    let open = is_open(&args.host, args.port, Duration::from_millis(args.timeout));
    let status = PortStatus {
        host: args.host.clone(),
        port: args.port,
        open,
    };
    output::write_or_json(
        global,
        || {
            println!(
                "{}:{} {}",
                args.host,
                args.port,
                if open { "open" } else { "closed" }
            );
            Ok(())
        },
        &status,
    )?;
    if open {
        Ok(())
    } else {
        Err(error::silent_exit(1))
    }
}

pub fn run_scan(args: &NetPortscanArgs, global: &GlobalOptions) -> Result<()> {
    if args.concurrency == 0 {
        return Err(error::msg("--concurrency must be greater than zero"));
    }
    let ports = parse_ports(&args.ports)?;
    let timeout = Duration::from_millis(args.timeout);
    let queue = Arc::new(Mutex::new(ports));
    let results = Arc::new(Mutex::new(Vec::<u16>::new()));
    let workers = args
        .concurrency
        .min(queue.lock().expect("queue lock").len())
        .max(1);

    thread::scope(|scope| {
        for _ in 0..workers {
            let queue = Arc::clone(&queue);
            let results = Arc::clone(&results);
            let host = args.host.clone();
            scope.spawn(move || {
                loop {
                    let port = {
                        let mut queue = queue.lock().expect("queue lock poisoned");
                        queue.pop()
                    };
                    let Some(port) = port else {
                        break;
                    };
                    if is_open(&host, port, timeout) {
                        results.lock().expect("results lock poisoned").push(port);
                    }
                }
            });
        }
    });

    let mut open_ports = Arc::try_unwrap(results)
        .expect("all workers joined")
        .into_inner()
        .expect("results lock poisoned");
    open_ports.sort_unstable();

    output::write_or_json(
        global,
        || {
            for port in &open_ports {
                println!("{port}");
            }
            Ok(())
        },
        &serde_json::json!({ "host": args.host, "open_ports": open_ports }),
    )
}

pub fn parse_ports(input: &str) -> Result<Vec<u16>> {
    let mut ports = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((start, end)) = part.split_once('-') {
            let start = parse_port(start)?;
            let end = parse_port(end)?;
            if start > end {
                return Err(error::msg("port range start must be <= end"));
            }
            ports.extend(start..=end);
        } else {
            ports.push(parse_port(part)?);
        }
    }
    ports.sort_unstable();
    ports.dedup();
    if ports.is_empty() {
        Err(error::msg("no ports were provided"))
    } else {
        Ok(ports)
    }
}

fn parse_port(value: &str) -> Result<u16> {
    value
        .trim()
        .parse::<u16>()
        .map_err(|_| error::msg(format!("invalid TCP port `{}`", value.trim())))
}

fn is_open(host: &str, port: u16, timeout: Duration) -> bool {
    let addrs = match (host, port).to_socket_addrs() {
        Ok(addrs) => addrs.collect::<Vec<SocketAddr>>(),
        Err(_) => return false,
    };
    addrs
        .into_iter()
        .any(|addr| TcpStream::connect_timeout(&addr, timeout).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ranges() {
        assert_eq!(parse_ports("80,443-445").unwrap(), vec![80, 443, 444, 445]);
    }
}
