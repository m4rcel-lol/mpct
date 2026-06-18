use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use data_encoding::{BASE32, BASE32_NOPAD};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use serde::Serialize;

use crate::{
    cli::{CodecArgs, CodecKind, GlobalOptions},
    error, output,
};

const URL_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}');

#[derive(Serialize)]
struct CodecOutput {
    codec: &'static str,
    output: String,
}

pub fn run_encode(args: &CodecArgs, global: &GlobalOptions) -> Result<()> {
    let input = output::read_input_bytes(&args.text, &args.file)?;
    let result = match args.codec {
        CodecKind::Base64 => {
            if args.url_safe {
                general_purpose::URL_SAFE_NO_PAD.encode(input)
            } else {
                general_purpose::STANDARD.encode(input)
            }
        }
        CodecKind::Hex => hex::encode(input),
        CodecKind::Url => {
            let text = String::from_utf8(input)
                .map_err(|err| error::msg(format!("URL encoding requires UTF-8 input: {err}")))?;
            utf8_percent_encode(&text, URL_ENCODE_SET).to_string()
        }
        CodecKind::Base32 => BASE32_NOPAD.encode(&input),
    };
    print_codec(args.codec, result, global)
}

pub fn run_decode(args: &CodecArgs, global: &GlobalOptions) -> Result<()> {
    let input = output::read_input_string(&args.text, &args.file)?;
    let bytes = match args.codec {
        CodecKind::Base64 => {
            let engine = if args.url_safe {
                &general_purpose::URL_SAFE_NO_PAD
            } else {
                &general_purpose::STANDARD
            };
            engine
                .decode(input.trim())
                .map_err(|err| error::msg(format!("invalid base64 input: {err}")))?
        }
        CodecKind::Hex => hex::decode(input.trim())
            .map_err(|err| error::msg(format!("invalid hex input: {err}")))?,
        CodecKind::Url => percent_decode_str(input.trim())
            .decode_utf8()
            .map_err(|err| error::msg(format!("invalid percent-encoded UTF-8: {err}")))?
            .into_owned()
            .into_bytes(),
        CodecKind::Base32 => {
            let normalized = input.trim().to_ascii_uppercase();
            BASE32_NOPAD
                .decode(normalized.as_bytes())
                .or_else(|_| BASE32.decode(normalized.as_bytes()))
                .map_err(|err| error::msg(format!("invalid base32 input: {err}")))?
        }
    };
    let result = String::from_utf8(bytes)
        .map_err(|err| error::msg(format!("decoded bytes are not valid UTF-8: {err}")))?;
    print_codec(args.codec, result, global)
}

fn print_codec(codec: CodecKind, result: String, global: &GlobalOptions) -> Result<()> {
    output::write_or_json(
        global,
        || {
            println!("{result}");
            Ok(())
        },
        &CodecOutput {
            codec: codec_name(codec),
            output: result.clone(),
        },
    )
}

fn codec_name(codec: CodecKind) -> &'static str {
    match codec {
        CodecKind::Base64 => "base64",
        CodecKind::Hex => "hex",
        CodecKind::Url => "url",
        CodecKind::Base32 => "base32",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_round_trip() {
        let encoded = utf8_percent_encode("a b?", URL_ENCODE_SET).to_string();
        assert_eq!(encoded, "a%20b%3F");
    }
}
