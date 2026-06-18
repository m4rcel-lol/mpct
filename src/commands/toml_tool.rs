use anyhow::Result;
use serde_json::Value as JsonValue;

use crate::{
    cli::{GlobalOptions, TomlCommand},
    commands::json_tool,
    error, output,
};

pub fn run(cmd: &TomlCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        TomlCommand::Pretty(args) => {
            let value = parse_toml(&args.file)?;
            println!("{}", toml::to_string_pretty(&value)?);
            Ok(())
        }
        TomlCommand::Validate(args) => {
            parse_toml(&args.file)?;
            if global.json {
                output::print_json(&serde_json::json!({ "valid": true }))
            } else {
                println!("VALID");
                Ok(())
            }
        }
        TomlCommand::ToJson(args) => {
            let value = parse_toml(&args.file)?;
            println!("{}", json_tool::json_pretty(&value, 2)?);
            Ok(())
        }
        TomlCommand::FromJson(args) => {
            let text = output::read_file_or_stdin(&args.file)?;
            let value: JsonValue = serde_json::from_str(&text)
                .map_err(|err| error::msg(format!("invalid JSON: {err}")))?;
            println!("{}", toml::to_string_pretty(&value)?);
            Ok(())
        }
    }
}

fn parse_toml(file: &Option<std::path::PathBuf>) -> Result<toml::Value> {
    let text = output::read_file_or_stdin(file)?;
    toml::from_str(&text).map_err(|err| error::msg(format!("invalid TOML: {err}")))
}
