use anyhow::Result;
use serde::Serialize;
use sha1::Digest as _;

use crate::{
    cli::{GlobalOptions, HashAlgo, HashArgs},
    error, output,
};

#[derive(Debug, Serialize)]
struct HashOutput<'a> {
    algorithm: &'a str,
    digest: String,
}

pub fn run(args: &HashArgs, global: &GlobalOptions) -> Result<()> {
    if args.args.first().is_some_and(|arg| arg == "verify") {
        verify(args, global)
    } else {
        compute(args, global)
    }
}

fn compute(args: &HashArgs, global: &GlobalOptions) -> Result<()> {
    if args.args.len() > 2 {
        return Err(error::msg("usage: mpct hash <algo> [TEXT] [--file <path>]"));
    }
    let algo = parse_algo(&args.args[0])?;
    let text = args.args.get(1).cloned();
    let input = output::read_input_bytes(&text, &args.file)?;
    let digest = digest_bytes(algo, &input);
    output::write_or_json(
        global,
        || {
            println!("{digest}");
            Ok(())
        },
        &HashOutput {
            algorithm: algo_name(algo),
            digest: digest.clone(),
        },
    )
}

fn verify(args: &HashArgs, global: &GlobalOptions) -> Result<()> {
    if !(3..=4).contains(&args.args.len()) {
        return Err(error::msg(
            "usage: mpct hash verify <algo> <expected_digest> [TEXT] [--file <path>]",
        ));
    }
    let algo = parse_algo(&args.args[1])?;
    let expected = args.args[2].to_ascii_lowercase();
    let text = args.args.get(3).cloned();
    let input = output::read_input_bytes(&text, &args.file)?;
    let digest = digest_bytes(algo, &input);
    let matched = digest.eq_ignore_ascii_case(&expected);

    if global.json {
        #[derive(Serialize)]
        struct VerifyOutput<'a> {
            algorithm: &'a str,
            expected: &'a str,
            actual: &'a str,
            matched: bool,
        }
        output::print_json(&VerifyOutput {
            algorithm: algo_name(algo),
            expected: &expected,
            actual: &digest,
            matched,
        })?;
    } else {
        println!("{}", if matched { "MATCH" } else { "MISMATCH" });
    }

    if matched {
        Ok(())
    } else {
        Err(error::silent_exit(1))
    }
}

pub fn parse_algo(value: &str) -> Result<HashAlgo> {
    match value.to_ascii_lowercase().as_str() {
        "md5" => Ok(HashAlgo::Md5),
        "sha1" => Ok(HashAlgo::Sha1),
        "sha256" => Ok(HashAlgo::Sha256),
        "sha512" => Ok(HashAlgo::Sha512),
        "blake3" => Ok(HashAlgo::Blake3),
        _ => Err(error::msg(format!(
            "unsupported hash algorithm `{value}`; expected md5, sha1, sha256, sha512, or blake3"
        ))),
    }
}

pub fn digest_bytes(algo: HashAlgo, input: &[u8]) -> String {
    match algo {
        HashAlgo::Md5 => hex::encode(md5::Md5::digest(input)),
        HashAlgo::Sha1 => hex::encode(sha1::Sha1::digest(input)),
        HashAlgo::Sha256 => hex::encode(sha2::Sha256::digest(input)),
        HashAlgo::Sha512 => hex::encode(sha2::Sha512::digest(input)),
        HashAlgo::Blake3 => blake3::hash(input).to_hex().to_string(),
    }
}

pub fn algo_name(algo: HashAlgo) -> &'static str {
    match algo {
        HashAlgo::Md5 => "md5",
        HashAlgo::Sha1 => "sha1",
        HashAlgo::Sha256 => "sha256",
        HashAlgo::Sha512 => "sha512",
        HashAlgo::Blake3 => "blake3",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vector() {
        assert_eq!(
            digest_bytes(HashAlgo::Sha256, b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn md5_known_vector() {
        assert_eq!(
            digest_bytes(HashAlgo::Md5, b"abc"),
            "900150983cd24fb0d6963f7d28e17f72"
        );
    }
}
