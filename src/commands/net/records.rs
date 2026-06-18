use anyhow::Result;

use crate::{
    cli::{DnsRecordType, GlobalOptions, NetDkimArgs, NetDomainArgs},
    commands::net::dns,
    error, output,
};

pub fn run_spf(args: &NetDomainArgs, global: &GlobalOptions) -> Result<()> {
    let records = txt_values(&args.domain)?;
    let spf = records
        .into_iter()
        .filter(|value| {
            normalize_txt(value)
                .to_ascii_lowercase()
                .starts_with("v=spf1")
        })
        .collect::<Vec<_>>();
    print_checked("spf", &args.domain, spf, global)
}

pub fn run_dkim(args: &NetDkimArgs, global: &GlobalOptions) -> Result<()> {
    let name = format!("{}._domainkey.{}", args.selector, args.domain);
    let records = txt_values(&name)?;
    let dkim = records
        .into_iter()
        .filter(|value| {
            normalize_txt(value)
                .to_ascii_lowercase()
                .starts_with("v=dkim1")
        })
        .collect::<Vec<_>>();
    print_checked("dkim", &name, dkim, global)
}

pub fn run_dmarc(args: &NetDomainArgs, global: &GlobalOptions) -> Result<()> {
    let name = format!("_dmarc.{}", args.domain);
    let records = txt_values(&name)?;
    let dmarc = records
        .into_iter()
        .filter(|value| {
            normalize_txt(value)
                .to_ascii_lowercase()
                .starts_with("v=dmarc1")
        })
        .collect::<Vec<_>>();
    print_checked("dmarc", &name, dmarc, global)
}

fn txt_values(domain: &str) -> Result<Vec<String>> {
    Ok(dns::lookup_records(domain, DnsRecordType::Txt)?
        .into_iter()
        .map(|record| record.value)
        .collect())
}

fn print_checked(
    kind: &str,
    name: &str,
    records: Vec<String>,
    global: &GlobalOptions,
) -> Result<()> {
    let valid = !records.is_empty();
    output::write_or_json(
        global,
        || {
            if valid {
                for record in &records {
                    println!("{record}");
                }
            } else {
                println!("NO {kind} record found for {name}");
            }
            Ok(())
        },
        &serde_json::json!({ "type": kind, "name": name, "valid": valid, "records": records }),
    )?;
    if valid {
        Ok(())
    } else {
        Err(error::silent_exit(1))
    }
}

fn normalize_txt(value: &str) -> String {
    value.replace('"', "").trim().to_string()
}
