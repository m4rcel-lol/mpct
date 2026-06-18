use anyhow::Result;
use serde_json::Value;

use crate::{
    cli::{GlobalOptions, YamlCommand},
    commands::json_tool,
    error, output,
};

pub fn run(cmd: &YamlCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        YamlCommand::Pretty(args) => {
            let value = parse_yaml(&args.file)?;
            let rendered = serde_yaml_ng::to_string(&value)
                .map_err(|err| error::msg(format!("failed to render YAML: {err}")))?;
            println!("{rendered}");
            Ok(())
        }
        YamlCommand::Minify(args) => {
            let value = parse_yaml(&args.file)?;
            println!("{}", serde_json::to_string(&value)?);
            Ok(())
        }
        YamlCommand::Validate(args) => {
            parse_yaml(&args.file)?;
            if global.json {
                output::print_json(&serde_json::json!({ "valid": true }))
            } else {
                println!("VALID");
                Ok(())
            }
        }
        YamlCommand::ToJson(args) => {
            let value = parse_yaml(&args.file)?;
            println!("{}", json_tool::json_pretty(&value, 2)?);
            Ok(())
        }
        YamlCommand::FromJson(args) => {
            let text = output::read_file_or_stdin(&args.file)?;
            let value: Value = serde_json::from_str(&text)
                .map_err(|err| error::msg(format!("invalid JSON: {err}")))?;
            let rendered = serde_yaml_ng::to_string(&value)
                .map_err(|err| error::msg(format!("failed to render YAML: {err}")))?;
            println!("{rendered}");
            Ok(())
        }
    }
}

fn parse_yaml(file: &Option<std::path::PathBuf>) -> Result<Value> {
    let text = output::read_file_or_stdin(file)?;
    serde_yaml_ng::from_str(&text).map_err(|err| error::msg(format!("invalid YAML: {err}")))
}
