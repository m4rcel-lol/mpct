use anyhow::Result;
use serde::Serialize;

use crate::{
    cli::{GlobalOptions, TextCommand},
    error, output,
};

pub fn run(cmd: &TextCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        TextCommand::Lorem(args) => {
            if args.paragraphs == 0 || args.words_per_paragraph == 0 {
                return Err(error::msg(
                    "paragraphs and words-per-paragraph must be greater than zero",
                ));
            }
            let paragraphs = (0..args.paragraphs)
                .map(|_| lipsum::lipsum_words(args.words_per_paragraph))
                .collect::<Vec<_>>();
            if global.json {
                output::print_json(&serde_json::json!({ "paragraphs": paragraphs }))
            } else {
                println!("{}", paragraphs.join("\n\n"));
                Ok(())
            }
        }
        TextCommand::Count(args) => {
            let text = output::read_file_or_stdin(&args.file)?;
            let count = count_text(&text);
            output::write_or_json(
                global,
                || {
                    println!("words: {}", count.words);
                    println!("lines: {}", count.lines);
                    println!("chars: {}", count.chars);
                    println!("bytes: {}", count.bytes);
                    Ok(())
                },
                &count,
            )
        }
        TextCommand::Slug(args) => {
            let slug = slugify(&args.text);
            output::write_or_json(
                global,
                || {
                    println!("{slug}");
                    Ok(())
                },
                &serde_json::json!({ "slug": slug }),
            )
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TextCount {
    words: usize,
    lines: usize,
    chars: usize,
    bytes: usize,
}

pub fn count_text(text: &str) -> TextCount {
    TextCount {
        words: text.split_whitespace().count(),
        lines: text.lines().count(),
        chars: text.chars().count(),
        bytes: text.len(),
    }
}

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_is_lowercase_hyphenated() {
        assert_eq!(slugify("Hello, MPCT!"), "hello-mpct");
    }
}
