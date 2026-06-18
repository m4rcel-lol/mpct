use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use data_encoding::BASE32_NOPAD;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde::Serialize;
use totp_rs::{Algorithm, Secret, TOTP};

use crate::{
    cli::{GlobalOptions, TotpCommand},
    error, output,
};

const URI_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'&')
    .add(b'+')
    .add(b'/')
    .add(b':')
    .add(b'<')
    .add(b'>')
    .add(b'=')
    .add(b'?');

pub fn run(cmd: &TotpCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        TotpCommand::New(args) => {
            let mut secret = [0_u8; 20];
            rand::fill(&mut secret);
            let encoded = BASE32_NOPAD.encode(&secret);
            if global.json {
                let uri = args
                    .uri
                    .then(|| provisioning_uri(&encoded, &args.issuer, &args.account, 6, 30));
                output::print_json(&serde_json::json!({ "secret": encoded, "uri": uri }))
            } else {
                println!("{encoded}");
                if args.uri {
                    println!(
                        "{}",
                        provisioning_uri(&encoded, &args.issuer, &args.account, 6, 30)
                    );
                }
                Ok(())
            }
        }
        TotpCommand::Gen(args) => {
            let totp = build_totp(&args.secret, args.digits, args.period, 0)?;
            let code = totp
                .generate_current()
                .map_err(|err| error::msg(format!("failed to read system time: {err}")))?;
            let remaining = totp
                .ttl()
                .map_err(|err| error::msg(format!("failed to read system time: {err}")))?;
            #[derive(Serialize)]
            struct GenOutput {
                code: String,
                seconds_remaining: u64,
            }
            output::write_or_json(
                global,
                || {
                    println!("{code}");
                    if !global.quiet {
                        println!("seconds remaining: {remaining}");
                    }
                    Ok(())
                },
                &GenOutput {
                    code: code.clone(),
                    seconds_remaining: remaining,
                },
            )
        }
        TotpCommand::Verify(args) => {
            let totp = build_totp(&args.secret, args.code.len(), 30, args.window)?;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|err| error::msg(format!("system clock is before Unix epoch: {err}")))?
                .as_secs();
            let valid = totp.check(&args.code, now);
            if global.json {
                output::print_json(&serde_json::json!({ "valid": valid }))?;
            } else {
                println!("{}", if valid { "VALID" } else { "INVALID" });
            }
            if valid {
                Ok(())
            } else {
                Err(error::silent_exit(1))
            }
        }
    }
}

fn build_totp(secret: &str, digits: usize, period: u64, skew: u8) -> Result<TOTP> {
    let secret = Secret::Encoded(secret.trim().to_ascii_uppercase())
        .to_bytes()
        .map_err(|err| error::msg(format!("invalid base32 TOTP secret: {err}")))?;
    TOTP::new(Algorithm::SHA1, digits, skew, period, secret)
        .map_err(|err| error::msg(format!("invalid TOTP parameters: {err}")))
}

fn provisioning_uri(
    secret: &str,
    issuer: &str,
    account: &str,
    digits: usize,
    period: u64,
) -> String {
    let issuer_enc = utf8_percent_encode(issuer, URI_SET);
    let account_enc = utf8_percent_encode(account, URI_SET);
    format!(
        "otpauth://totp/{issuer_enc}:{account_enc}?secret={secret}&issuer={issuer_enc}&algorithm=SHA1&digits={digits}&period={period}"
    )
}
