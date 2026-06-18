use anyhow::Result;
use nanoid::nanoid;
use serde::Serialize;
use ulid::Ulid;
use uuid::Uuid;

use crate::{
    cli::{CountArgs, GlobalOptions, NanoidArgs, UuidArgs},
    error, output,
};

#[derive(Serialize)]
struct IdOutput {
    values: Vec<String>,
}

pub fn run_uuid(args: &UuidArgs, global: &GlobalOptions) -> Result<()> {
    validate_count(args.count)?;
    let values = (0..args.count)
        .map(|_| {
            let value = if args.v7 {
                Uuid::now_v7()
            } else {
                Uuid::new_v4()
            };
            let mut value = value.to_string();
            if args.upper {
                value.make_ascii_uppercase();
            }
            value
        })
        .collect::<Vec<_>>();
    print_values(values, global)
}

pub fn run_ulid(args: &CountArgs, global: &GlobalOptions) -> Result<()> {
    validate_count(args.count)?;
    let values = (0..args.count)
        .map(|_| Ulid::new().to_string())
        .collect::<Vec<_>>();
    print_values(values, global)
}

pub fn run_nanoid(args: &NanoidArgs, global: &GlobalOptions) -> Result<()> {
    validate_count(args.count)?;
    if args.size == 0 {
        return Err(error::msg("--size must be greater than zero"));
    }
    let values = match &args.alphabet {
        Some(alphabet) => {
            let chars = alphabet.chars().collect::<Vec<_>>();
            if chars.len() < 2 {
                return Err(error::msg(
                    "--alphabet must contain at least two characters",
                ));
            }
            (0..args.count)
                .map(|_| nanoid!(args.size, &chars))
                .collect::<Vec<_>>()
        }
        None => (0..args.count)
            .map(|_| nanoid!(args.size))
            .collect::<Vec<_>>(),
    };
    print_values(values, global)
}

fn validate_count(count: usize) -> Result<()> {
    if count == 0 {
        Err(error::msg("count must be greater than zero"))
    } else {
        Ok(())
    }
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
        &IdOutput {
            values: values.clone(),
        },
    )
}
