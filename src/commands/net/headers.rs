use anyhow::Result;
use serde::Serialize;

use crate::{
    cli::{GlobalOptions, HttpMethod, NetHeadersArgs},
    error, output,
};

#[derive(Debug, Serialize)]
struct HeaderLine {
    name: String,
    value: String,
}

pub fn run(args: &NetHeadersArgs, global: &GlobalOptions) -> Result<()> {
    let response = match args.method {
        HttpMethod::Get => ureq::get(&args.url).call(),
        HttpMethod::Head => ureq::head(&args.url).call(),
        HttpMethod::Post => ureq::post(&args.url).send(""),
    }
    .map_err(|err| error::msg(format!("HTTP request failed: {err}")))?;
    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| HeaderLine {
            name: name.as_str().to_string(),
            value: value.to_str().unwrap_or("<non-UTF8>").to_string(),
        })
        .collect::<Vec<_>>();
    output::write_or_json(
        global,
        || {
            println!("status: {status}");
            for header in &headers {
                println!("{}: {}", header.name, header.value);
            }
            Ok(())
        },
        &serde_json::json!({ "status": status, "headers": headers }),
    )
}
