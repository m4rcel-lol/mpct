use anyhow::Result;
use regex::Regex;
use serde::Serialize;

use crate::{
    cli::{GlobalOptions, RegexCommand},
    error, output,
};

pub fn run(cmd: &RegexCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        RegexCommand::Test(args) => {
            let re = Regex::new(&args.pattern)
                .map_err(|err| error::msg(format!("invalid regex pattern: {err}")))?;
            let text = output::read_input_string(&args.text, &args.file)?;
            let result = test_regex(&re, &text);
            output::write_or_json(
                global,
                || {
                    println!("matched: {}", result.matched);
                    for mat in &result.matches {
                        println!("match {}..{}: {}", mat.start, mat.end, mat.text);
                    }
                    for cap in &result.named_captures {
                        println!("capture {}: {}", cap.name, cap.value);
                    }
                    Ok(())
                },
                &result,
            )
        }
        RegexCommand::Replace(args) => {
            let re = Regex::new(&args.pattern)
                .map_err(|err| error::msg(format!("invalid regex pattern: {err}")))?;
            let replaced = re
                .replace_all(&args.text, args.replacement.as_str())
                .to_string();
            output::write_or_json(
                global,
                || {
                    println!("{replaced}");
                    Ok(())
                },
                &serde_json::json!({ "output": replaced }),
            )
        }
    }
}

#[derive(Debug, Serialize)]
struct RegexResult {
    matched: bool,
    matches: Vec<MatchResult>,
    named_captures: Vec<NamedCapture>,
}

#[derive(Debug, Serialize)]
struct MatchResult {
    start: usize,
    end: usize,
    text: String,
}

#[derive(Debug, Serialize)]
struct NamedCapture {
    name: String,
    value: String,
}

fn test_regex(re: &Regex, text: &str) -> RegexResult {
    let matches = re
        .find_iter(text)
        .map(|mat| MatchResult {
            start: mat.start(),
            end: mat.end(),
            text: mat.as_str().to_string(),
        })
        .collect::<Vec<_>>();
    let mut named_captures = Vec::new();
    for captures in re.captures_iter(text) {
        for name in re.capture_names().flatten() {
            if let Some(value) = captures.name(name) {
                named_captures.push(NamedCapture {
                    name: name.to_string(),
                    value: value.as_str().to_string(),
                });
            }
        }
    }
    RegexResult {
        matched: !matches.is_empty(),
        matches,
        named_captures,
    }
}
