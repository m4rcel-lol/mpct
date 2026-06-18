use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

use crate::{
    cli::{GlobalOptions, JsonCommand},
    error, output,
};

pub fn run(cmd: &JsonCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        JsonCommand::Pretty(args) => {
            let value = parse_json(&args.file)?;
            println!("{}", json_pretty(&value, args.indent)?);
            Ok(())
        }
        JsonCommand::Minify(args) => {
            let value = parse_json(&args.file)?;
            println!("{}", serde_json::to_string(&value)?);
            Ok(())
        }
        JsonCommand::Validate(args) => {
            parse_json(&args.file)?;
            if global.json {
                output::print_json(&serde_json::json!({ "valid": true }))
            } else {
                println!("VALID");
                Ok(())
            }
        }
        JsonCommand::Query(args) => {
            let value = parse_json(&args.file)?;
            let selected = query_path(&value, &args.path_expr)?;
            if global.json {
                output::print_json(selected)
            } else if selected.is_string() {
                println!("{}", selected.as_str().expect("checked string"));
                Ok(())
            } else {
                println!("{}", json_pretty(selected, 2)?);
                Ok(())
            }
        }
    }
}

fn parse_json(file: &Option<std::path::PathBuf>) -> Result<Value> {
    let text = output::read_file_or_stdin(file)?;
    serde_json::from_str(&text).map_err(|err| error::msg(format!("invalid JSON: {err}")))
}

pub fn json_pretty<T: Serialize>(value: &T, indent: usize) -> Result<String> {
    if indent == 0 {
        return Err(error::msg("--indent must be greater than zero"));
    }
    let mut bytes = Vec::new();
    let indent = vec![b' '; indent];
    let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent);
    let mut serializer = serde_json::Serializer::with_formatter(&mut bytes, formatter);
    value.serialize(&mut serializer)?;
    String::from_utf8(bytes).map_err(|err| error::msg(format!("failed to render JSON: {err}")))
}

pub fn query_path<'a>(value: &'a Value, expr: &str) -> Result<&'a Value> {
    let expr = expr.strip_prefix('.').unwrap_or(expr);
    if expr.is_empty() {
        return Ok(value);
    }

    let mut current = value;
    for segment in expr.split('.') {
        let (name, index) = parse_segment(segment)?;
        current = current
            .get(name)
            .ok_or_else(|| error::msg(format!("path segment `{name}` was not found")))?;
        if let Some(index) = index {
            current = current
                .as_array()
                .and_then(|array| array.get(index))
                .ok_or_else(|| error::msg(format!("array index [{index}] was not found")))?;
        }
    }
    Ok(current)
}

fn parse_segment(segment: &str) -> Result<(&str, Option<usize>)> {
    if segment.is_empty() {
        return Err(error::msg("JSON path contains an empty segment"));
    }

    let Some(bracket_start) = segment.find('[') else {
        validate_identifier(segment)?;
        return Ok((segment, None));
    };
    if !segment.ends_with(']') {
        return Err(error::msg(format!("invalid JSON path segment `{segment}`")));
    }
    let name = &segment[..bracket_start];
    validate_identifier(name)?;
    let raw_index = &segment[bracket_start + 1..segment.len() - 1];
    let index = raw_index
        .parse::<usize>()
        .map_err(|_| error::msg(format!("invalid array index `{raw_index}`")))?;
    Ok((name, Some(index)))
}

fn validate_identifier(value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        Err(error::msg(format!(
            "invalid JSON path identifier `{value}`; expected [A-Za-z0-9_]+"
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_array_path() {
        let value = serde_json::json!({ "users": [{ "email": "a@example.com" }] });
        assert_eq!(
            query_path(&value, ".users[0].email").unwrap(),
            &Value::String("a@example.com".to_string())
        );
    }

    #[test]
    fn rejects_wildcards() {
        let value = serde_json::json!({ "users": [] });
        assert!(query_path(&value, ".users[*]").is_err());
    }
}
