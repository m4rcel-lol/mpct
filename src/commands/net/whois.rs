use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

use anyhow::Result;

use crate::{
    cli::{GlobalOptions, NetWhoisArgs},
    error, output,
};

pub fn run(args: &NetWhoisArgs, global: &GlobalOptions) -> Result<()> {
    let iana = query("whois.iana.org", &args.domain)?;
    let server = referral_server(&iana)
        .unwrap_or("whois.iana.org")
        .to_string();
    let response = if server == "whois.iana.org" {
        iana
    } else {
        query(&server, &args.domain)?
    };
    output::write_or_json(
        global,
        || {
            print!("{response}");
            Ok(())
        },
        &serde_json::json!({ "server": server, "response": response }),
    )
}

fn query(server: &str, domain: &str) -> Result<String> {
    let mut stream = TcpStream::connect((server, 43))
        .map_err(|err| error::msg(format!("failed to connect to WHOIS server {server}: {err}")))?;
    stream.set_read_timeout(Some(Duration::from_secs(15))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(15))).ok();
    stream
        .write_all(format!("{domain}\r\n").as_bytes())
        .map_err(|err| error::msg(format!("failed to query WHOIS server {server}: {err}")))?;
    let mut response = String::new();
    stream.read_to_string(&mut response).map_err(|err| {
        error::msg(format!(
            "failed to read WHOIS response from {server}: {err}"
        ))
    })?;
    Ok(response)
}

fn referral_server(response: &str) -> Option<&str> {
    response.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim().eq_ignore_ascii_case("whois") {
            Some(value.trim())
        } else {
            None
        }
    })
}
