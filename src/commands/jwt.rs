use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, KeyInit, Mac};
use serde::Serialize;
use serde_json::Value;
use sha2::Sha256;

use crate::{
    cli::{GlobalOptions, JwtCommand},
    error, output,
};

type HmacSha256 = Hmac<Sha256>;

pub fn run(cmd: &JwtCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        JwtCommand::Decode(args) => decode(&args.token, global),
        JwtCommand::Verify(args) => verify(&args.token, &args.secret, &args.alg, global),
    }
}

fn decode(token: &str, global: &GlobalOptions) -> Result<()> {
    let parts = split_token(token)?;
    let header = decode_json(parts[0], "header")?;
    let payload = decode_json(parts[1], "payload")?;
    if global.json {
        output::print_json(&serde_json::json!({
            "verified": false,
            "warning": "signature not verified",
            "header": header,
            "payload": payload
        }))
    } else {
        eprintln!("warning: signature not verified");
        println!("header:");
        println!("{}", crate::commands::json_tool::json_pretty(&header, 2)?);
        println!("payload:");
        println!("{}", crate::commands::json_tool::json_pretty(&payload, 2)?);
        Ok(())
    }
}

fn verify(token: &str, secret: &str, alg: &str, global: &GlobalOptions) -> Result<()> {
    if alg != "HS256" {
        return Err(error::msg("only HS256 JWT verification is supported"));
    }
    let parts = split_token(token)?;
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let actual_sig = decode_base64url(parts[2])?;
    let mut mac = <HmacSha256 as KeyInit>::new_from_slice(secret.as_bytes())
        .map_err(|err| error::msg(format!("invalid HMAC secret: {err}")))?;
    mac.update(signing_input.as_bytes());
    let valid = mac.verify_slice(&actual_sig).is_ok();
    let header = decode_json(parts[0], "header")?;
    let payload = decode_json(parts[1], "payload")?;

    #[derive(Serialize)]
    struct VerifyOutput {
        valid: bool,
        header: Value,
        claims: Value,
    }

    if global.json {
        output::print_json(&VerifyOutput {
            valid,
            header,
            claims: payload,
        })?;
    } else {
        println!("{}", if valid { "valid" } else { "invalid" });
        if valid && !global.quiet {
            println!("{}", crate::commands::json_tool::json_pretty(&payload, 2)?);
        }
    }

    if valid {
        Ok(())
    } else {
        Err(error::silent_exit(1))
    }
}

fn split_token(token: &str) -> Result<Vec<&str>> {
    let parts = token.split('.').collect::<Vec<_>>();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        Err(error::msg(
            "JWT must contain exactly three non-empty base64url parts",
        ))
    } else {
        Ok(parts)
    }
}

fn decode_json(part: &str, label: &str) -> Result<Value> {
    let bytes = decode_base64url(part)?;
    serde_json::from_slice(&bytes)
        .map_err(|err| error::msg(format!("invalid JWT {label} JSON: {err}")))
}

fn decode_base64url(part: &str) -> Result<Vec<u8>> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(part)
        .or_else(|_| general_purpose::URL_SAFE.decode(part))
        .map_err(|err| error::msg(format!("invalid base64url data: {err}")))
}
