use anyhow::Result;
use hickory_resolver::{Resolver, proto::rr::RecordType};
use serde::Serialize;
use tokio::runtime::Builder;

use crate::{
    cli::{DnsRecordType, GlobalOptions, NetDnsArgs},
    error, output,
};

#[derive(Debug, Serialize)]
pub struct DnsRecord {
    pub record_type: String,
    pub value: String,
}

pub fn run(args: &NetDnsArgs, global: &GlobalOptions) -> Result<()> {
    let records = lookup_records(&args.domain, args.record_type)?;
    output::write_or_json(
        global,
        || {
            for record in &records {
                println!("{} {}", record.record_type, record.value);
            }
            Ok(())
        },
        &serde_json::json!({ "domain": args.domain, "records": records }),
    )
}

pub fn lookup_records(domain: &str, record_type: DnsRecordType) -> Result<Vec<DnsRecord>> {
    let types = match record_type {
        DnsRecordType::All => vec![
            DnsRecordType::A,
            DnsRecordType::Aaaa,
            DnsRecordType::Mx,
            DnsRecordType::Txt,
            DnsRecordType::Ns,
            DnsRecordType::Cname,
        ],
        one => vec![one],
    };
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| error::msg(format!("failed to create DNS runtime: {err}")))?;
    let mut out = Vec::new();
    rt.block_on(async {
        let resolver = Resolver::builder_tokio()
            .map_err(|err| error::msg(format!("failed to load DNS resolver config: {err}")))?
            .build()
            .map_err(|err| error::msg(format!("failed to build DNS resolver: {err}")))?;
        for item in types {
            let record_type = to_hickory_type(item);
            match resolver.lookup(domain, record_type).await {
                Ok(lookup) => {
                    for record in lookup.answers() {
                        out.push(DnsRecord {
                            record_type: format!("{:?}", record.record_type()),
                            value: record.data.to_string(),
                        });
                    }
                }
                Err(err) if matches!(item, DnsRecordType::All) => {
                    out.push(DnsRecord {
                        record_type: format!("{record_type:?}"),
                        value: format!("lookup failed: {err}"),
                    });
                }
                Err(err) => return Err(error::msg(format!("DNS lookup failed: {err}"))),
            }
        }
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(out)
}

fn to_hickory_type(record_type: DnsRecordType) -> RecordType {
    match record_type {
        DnsRecordType::A => RecordType::A,
        DnsRecordType::Aaaa => RecordType::AAAA,
        DnsRecordType::Mx => RecordType::MX,
        DnsRecordType::Txt => RecordType::TXT,
        DnsRecordType::Ns => RecordType::NS,
        DnsRecordType::Cname => RecordType::CNAME,
        DnsRecordType::All => RecordType::A,
    }
}
