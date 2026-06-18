use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{
    cli::{ChecksumArgs, GlobalOptions, HashAlgo},
    commands::hash,
    error, output,
};

#[derive(Debug, Serialize)]
struct ChecksumLine {
    digest: String,
    path: String,
}

pub fn run(args: &ChecksumArgs, global: &GlobalOptions) -> Result<()> {
    if args
        .args
        .first()
        .is_some_and(|path| path == Path::new("verify"))
    {
        verify(args, global)
    } else {
        generate(args, global)
    }
}

fn generate(args: &ChecksumArgs, global: &GlobalOptions) -> Result<()> {
    let files = collect_files(&args.args, args.recursive)?;
    if files.is_empty() {
        return Err(error::msg("no files matched the checksum request"));
    }

    let mut lines = Vec::with_capacity(files.len());
    for file in files {
        let bytes =
            fs::read(&file).with_context(|| format!("failed to read file {}", file.display()))?;
        lines.push(ChecksumLine {
            digest: hash::digest_bytes(HashAlgo::Sha256, &bytes),
            path: file.display().to_string(),
        });
    }

    output::write_or_json(
        global,
        || {
            for line in &lines {
                println!("{}  {}", line.digest, line.path);
            }
            Ok(())
        },
        &lines,
    )
}

fn verify(args: &ChecksumArgs, global: &GlobalOptions) -> Result<()> {
    if args.args.len() != 2 {
        return Err(error::msg("usage: mpct checksum verify <manifest_file>"));
    }
    let manifest = &args.args[1];
    let contents = fs::read_to_string(manifest)
        .with_context(|| format!("failed to read manifest {}", manifest.display()))?;

    #[derive(Serialize)]
    struct VerifyLine {
        path: String,
        expected: String,
        actual: Option<String>,
        ok: bool,
        error: Option<String>,
    }

    let mut results = Vec::new();
    for (idx, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        let (expected, path) = parse_manifest_line(line).ok_or_else(|| {
            error::msg(format!(
                "invalid manifest line {}: expected `<digest>  <path>`",
                idx + 1
            ))
        })?;
        match fs::read(path) {
            Ok(bytes) => {
                let actual = hash::digest_bytes(HashAlgo::Sha256, &bytes);
                let ok = actual.eq_ignore_ascii_case(expected);
                results.push(VerifyLine {
                    path: path.to_string(),
                    expected: expected.to_string(),
                    actual: Some(actual),
                    ok,
                    error: None,
                });
            }
            Err(err) => results.push(VerifyLine {
                path: path.to_string(),
                expected: expected.to_string(),
                actual: None,
                ok: false,
                error: Some(err.to_string()),
            }),
        }
    }

    let all_ok = results.iter().all(|line| line.ok);
    if global.json {
        output::print_json(&results)?;
    } else {
        for line in &results {
            println!("{}: {}", line.path, if line.ok { "OK" } else { "FAILED" });
        }
    }

    if all_ok {
        Ok(())
    } else {
        Err(error::silent_exit(1))
    }
}

fn parse_manifest_line(line: &str) -> Option<(&str, &str)> {
    let mut split = line.splitn(2, char::is_whitespace);
    let digest = split.next()?.trim();
    let path = split.next()?.trim();
    if digest.is_empty() || path.is_empty() {
        None
    } else {
        Some((digest, path))
    }
}

fn collect_files(paths: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            if recursive {
                collect_dir(path, &mut files)?;
            } else {
                return Err(error::msg(format!(
                    "{} is a directory; pass --recursive to include it",
                    path.display()
                )));
            }
        } else {
            return Err(error::msg(format!(
                "{} is not a regular file",
                path.display()
            )));
        }
    }
    files.sort();
    Ok(files)
}

fn collect_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in
        fs::read_dir(dir).with_context(|| format!("failed to read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_dir(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sha_style_line() {
        assert_eq!(
            parse_manifest_line("abc123  path/to/file.txt"),
            Some(("abc123", "path/to/file.txt"))
        );
    }
}
