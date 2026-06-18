use anyhow::Result;
use serde::Serialize;
use zxcvbn::Score;

use crate::{
    cli::{GlobalOptions, PassCommand},
    error, output,
};

const WORDLIST_TEXT: &str = include_str!("pass_wordlist.txt");

pub fn run(cmd: &PassCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        PassCommand::Gen(args) => {
            if args.length == 0 || args.count == 0 {
                return Err(error::msg("length and count must be greater than zero"));
            }
            let values = (0..args.count)
                .map(|_| generate_password(args.length, args.no_symbols, args.no_digits))
                .collect::<Result<Vec<_>>>()?;
            print_values(values, global)
        }
        PassCommand::Phrase(args) => {
            if args.words == 0 || args.count == 0 {
                return Err(error::msg("words and count must be greater than zero"));
            }
            let words = wordlist();
            if words.len() < 1000 {
                return Err(error::msg("embedded passphrase wordlist is too small"));
            }
            let values = (0..args.count)
                .map(|_| {
                    (0..args.words)
                        .map(|_| words[rand::random_range(0..words.len())])
                        .collect::<Vec<_>>()
                        .join(&args.separator)
                })
                .collect::<Vec<_>>();
            print_values(values, global)
        }
        PassCommand::Strength(args) => {
            let entropy = zxcvbn::zxcvbn(&args.password, &[]);
            let score = score_to_u8(entropy.score());
            let label = strength_label(score);
            #[derive(Serialize)]
            struct StrengthOutput<'a> {
                score: u8,
                strength: &'a str,
                guesses: u64,
            }
            output::write_or_json(
                global,
                || {
                    println!("score: {score}/4");
                    println!("strength: {label}");
                    Ok(())
                },
                &StrengthOutput {
                    score,
                    strength: label,
                    guesses: entropy.guesses(),
                },
            )
        }
    }
}

fn generate_password(length: usize, no_symbols: bool, no_digits: bool) -> Result<String> {
    let mut alphabet = String::from("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
    if !no_digits {
        alphabet.push_str("0123456789");
    }
    if !no_symbols {
        alphabet.push_str("!@#$%^&*()-_=+[]{};:,.?/|~");
    }
    let chars = alphabet.chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return Err(error::msg("password alphabet is empty"));
    }
    Ok((0..length)
        .map(|_| chars[rand::random_range(0..chars.len())])
        .collect())
}

fn wordlist() -> Vec<&'static str> {
    WORDLIST_TEXT.split_whitespace().collect()
}

fn print_values(values: Vec<String>, global: &GlobalOptions) -> Result<()> {
    output::write_or_json(
        global,
        || {
            for value in &values {
                println!("{value}");
            }
            Ok(())
        },
        &serde_json::json!({ "values": values }),
    )
}

fn score_to_u8(score: Score) -> u8 {
    score.into()
}

fn strength_label(score: u8) -> &'static str {
    match score {
        0 | 1 => "weak",
        2 => "fair",
        3 => "strong",
        _ => "excellent",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wordlist_is_adequate_size() {
        assert!(wordlist().len() >= 1000);
    }
}
