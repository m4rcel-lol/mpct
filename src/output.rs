use std::{
    fs,
    io::{self, IsTerminal, Read},
    path::Path,
};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::cli::GlobalOptions;

pub fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub fn read_input_bytes(
    text: &Option<String>,
    file: &Option<std::path::PathBuf>,
) -> Result<Vec<u8>> {
    match (text, file) {
        (Some(_), Some(_)) => Err(crate::error::msg("provide either TEXT or --file, not both")),
        (Some(text), None) => Ok(text.as_bytes().to_vec()),
        (None, Some(path)) => {
            fs::read(path).with_context(|| format!("failed to read file {}", path.display()))
        }
        (None, None) => {
            let mut buf = Vec::new();
            io::stdin()
                .read_to_end(&mut buf)
                .context("failed to read stdin")?;
            Ok(buf)
        }
    }
}

pub fn read_input_string(
    text: &Option<String>,
    file: &Option<std::path::PathBuf>,
) -> Result<String> {
    let bytes = read_input_bytes(text, file)?;
    String::from_utf8(bytes)
        .map_err(|err| crate::error::msg(format!("input is not valid UTF-8: {err}")))
}

pub fn read_file_or_stdin(file: &Option<std::path::PathBuf>) -> Result<String> {
    read_input_string(&None, file)
}

pub fn write_or_json<T>(
    global: &GlobalOptions,
    human: impl FnOnce() -> Result<()>,
    json: &T,
) -> Result<()>
where
    T: Serialize,
{
    if global.json {
        print_json(json)
    } else {
        human()
    }
}

pub fn color_enabled(global: &GlobalOptions) -> bool {
    !global.no_color && std::env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal()
}

pub fn overwrite_confirm(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    eprint!("Overwrite {}? [y/N] ", path.display());
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .context("failed to read overwrite confirmation")?;
    match answer.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => Ok(()),
        _ => Err(crate::error::msg("operation cancelled by user")),
    }
}
