use std::{net::TcpStream, sync::Arc, time::Duration};

use anyhow::Result;
use chrono::{TimeZone, Utc};
use rustls::{ClientConfig, ClientConnection, RootCertStore, pki_types::ServerName};
use serde::Serialize;
use x509_parser::{extensions::GeneralName, prelude::*};

use crate::{
    cli::{GlobalOptions, NetTlsArgs},
    error, output,
};

#[derive(Debug, Serialize)]
struct TlsInfo {
    host: String,
    port: u16,
    subject: String,
    issuer: String,
    not_before: String,
    not_after: String,
    days_until_expiry: i64,
    sans: Vec<String>,
}

pub fn run(args: &NetTlsArgs, global: &GlobalOptions) -> Result<()> {
    let (host, port) = parse_host_port(&args.host)?;
    let info = inspect(&host, port)?;
    output::write_or_json(
        global,
        || {
            println!("subject: {}", info.subject);
            println!("issuer: {}", info.issuer);
            println!("not before: {}", info.not_before);
            println!("not after: {}", info.not_after);
            println!("days until expiry: {}", info.days_until_expiry);
            println!("SANs: {}", info.sans.join(", "));
            Ok(())
        },
        &info,
    )
}

fn inspect(host: &str, port: u16) -> Result<TlsInfo> {
    let mut root_store = RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs();
    let (added, _) = root_store.add_parsable_certificates(certs.certs);
    if added == 0 {
        return Err(error::msg(
            "no native TLS root certificates could be loaded",
        ));
    }
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let server_name = ServerName::try_from(host.to_string())
        .map_err(|_| error::msg(format!("invalid TLS server name `{host}`")))?;
    let mut conn = ClientConnection::new(Arc::new(config), server_name)
        .map_err(|err| error::msg(format!("failed to create TLS client: {err}")))?;
    let mut stream = TcpStream::connect((host, port))
        .map_err(|err| error::msg(format!("failed to connect to {host}:{port}: {err}")))?;
    stream.set_read_timeout(Some(Duration::from_secs(15))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(15))).ok();
    while conn.is_handshaking() {
        conn.complete_io(&mut stream)
            .map_err(|err| error::msg(format!("TLS handshake failed: {err}")))?;
    }
    let certs = conn
        .peer_certificates()
        .ok_or_else(|| error::msg("server did not provide a certificate chain"))?;
    let leaf = certs
        .first()
        .ok_or_else(|| error::msg("server certificate chain is empty"))?;
    let (_, cert) = X509Certificate::from_der(leaf.as_ref())
        .map_err(|err| error::msg(format!("failed to parse leaf certificate: {err}")))?;
    let validity = cert.validity();
    let now = Utc::now().timestamp();
    let not_before = Utc
        .timestamp_opt(validity.not_before.timestamp(), 0)
        .single()
        .ok_or_else(|| error::msg("certificate notBefore is outside supported range"))?
        .to_rfc3339();
    let not_after_dt = Utc
        .timestamp_opt(validity.not_after.timestamp(), 0)
        .single()
        .ok_or_else(|| error::msg("certificate notAfter is outside supported range"))?;
    let days_until_expiry = (validity.not_after.timestamp() - now) / 86_400;
    let sans = cert
        .subject_alternative_name()
        .ok()
        .flatten()
        .map(|san| {
            san.value
                .general_names
                .iter()
                .map(format_general_name)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(TlsInfo {
        host: host.to_string(),
        port,
        subject: cert.subject().to_string(),
        issuer: cert.issuer().to_string(),
        not_before,
        not_after: not_after_dt.to_rfc3339(),
        days_until_expiry,
        sans,
    })
}

fn parse_host_port(input: &str) -> Result<(String, u16)> {
    if let Some(rest) = input.strip_prefix('[') {
        let (host, after) = rest
            .split_once(']')
            .ok_or_else(|| error::msg("invalid bracketed IPv6 host"))?;
        let port = after
            .strip_prefix(':')
            .map(parse_port)
            .transpose()?
            .unwrap_or(443);
        return Ok((host.to_string(), port));
    }
    if let Some((host, port)) = input.rsplit_once(':')
        && !host.contains(':')
    {
        return Ok((host.to_string(), parse_port(port)?));
    }
    Ok((input.to_string(), 443))
}

fn parse_port(value: &str) -> Result<u16> {
    value
        .parse::<u16>()
        .map_err(|_| error::msg(format!("invalid TLS port `{value}`")))
}

fn format_general_name(name: &GeneralName<'_>) -> String {
    match name {
        GeneralName::DNSName(value) => format!("DNS:{value}"),
        GeneralName::IPAddress(bytes) if bytes.len() == 4 => {
            format!("IP:{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
        }
        GeneralName::IPAddress(bytes) if bytes.len() == 16 => {
            let mut octets = [0_u8; 16];
            octets.copy_from_slice(bytes);
            format!("IP:{}", std::net::Ipv6Addr::from(octets))
        }
        GeneralName::RFC822Name(value) => format!("email:{value}"),
        GeneralName::URI(value) => format!("URI:{value}"),
        other => format!("{other:?}"),
    }
}
