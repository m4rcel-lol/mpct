use anyhow::Result;

use crate::{
    cli::{GlobalOptions, NetMyIpArgs},
    error, output,
};

pub fn run(args: &NetMyIpArgs, global: &GlobalOptions) -> Result<()> {
    let url = if args.v6 {
        "https://api6.ipify.org"
    } else {
        "https://api.ipify.org"
    };
    let mut response = ureq::get(url)
        .call()
        .map_err(|err| error::msg(format!("failed to query public IP service: {err}")))?;
    let ip = response
        .body_mut()
        .read_to_string()
        .map_err(|err| error::msg(format!("failed to read public IP response: {err}")))?
        .trim()
        .to_string();
    output::write_or_json(
        global,
        || {
            println!("{ip}");
            Ok(())
        },
        &serde_json::json!({ "ip": ip }),
    )
}
