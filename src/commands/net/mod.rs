pub mod dns;
pub mod headers;
pub mod myip;
pub mod ping;
pub mod portscan;
pub mod records;
pub mod tls;
pub mod whois;

use anyhow::Result;

use crate::{
    cli::{GlobalOptions, NetCommand},
    commands::net,
};

pub fn run(cmd: &NetCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        NetCommand::Ping(args) => net::ping::run(args, global),
        NetCommand::Port(args) => net::portscan::run_port(args, global),
        NetCommand::Portscan(args) => net::portscan::run_scan(args, global),
        NetCommand::Dns(args) => net::dns::run(args, global),
        NetCommand::Whois(args) => net::whois::run(args, global),
        NetCommand::Tls(args) => net::tls::run(args, global),
        NetCommand::Headers(args) => net::headers::run(args, global),
        NetCommand::Myip(args) => net::myip::run(args, global),
        NetCommand::Spf(args) => net::records::run_spf(args, global),
        NetCommand::Dkim(args) => net::records::run_dkim(args, global),
        NetCommand::Dmarc(args) => net::records::run_dmarc(args, global),
    }
}
