pub mod banner;
pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod output;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use crate::{
    banner::print_banner,
    cli::{Cli, Commands},
};

pub fn run() -> Result<()> {
    run_with_args(std::env::args())
}

pub fn run_with_args<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let args_vec: Vec<std::ffi::OsString> = args.into_iter().map(Into::into).collect();
    if is_top_level_help(&args_vec) {
        let no_banner = args_vec.iter().any(|arg| arg == "--no-banner");
        if !no_banner {
            print_banner(false);
        }
        Cli::command().print_long_help()?;
        println!();
        return Ok(());
    }

    let cli = match Cli::try_parse_from(args_vec) {
        Ok(cli) => cli,
        Err(err) => err.exit(),
    };

    let result = match &cli.command {
        None => {
            if !cli.global.no_banner {
                print_banner(false);
            }
            print_usage_hint();
            Ok(())
        }
        Some(Commands::Banner) => {
            if !cli.global.no_banner {
                print_banner(true);
            }
            Ok(())
        }
        Some(Commands::Hash(args)) => commands::hash::run(args, &cli.global),
        Some(Commands::Checksum(args)) => commands::checksum::run(args, &cli.global),
        Some(Commands::Encode(args)) => commands::codec::run_encode(args, &cli.global),
        Some(Commands::Decode(args)) => commands::codec::run_decode(args, &cli.global),
        Some(Commands::Jwt(cmd)) => commands::jwt::run(cmd, &cli.global),
        Some(Commands::Uuid(args)) => commands::ids::run_uuid(args, &cli.global),
        Some(Commands::Ulid(args)) => commands::ids::run_ulid(args, &cli.global),
        Some(Commands::Nanoid(args)) => commands::ids::run_nanoid(args, &cli.global),
        Some(Commands::Pass(cmd)) => commands::pass::run(cmd, &cli.global),
        Some(Commands::Totp(cmd)) => commands::totp::run(cmd, &cli.global),
        Some(Commands::Json(cmd)) => commands::json_tool::run(cmd, &cli.global),
        Some(Commands::Yaml(cmd)) => commands::yaml_tool::run(cmd, &cli.global),
        Some(Commands::Toml(cmd)) => commands::toml_tool::run(cmd, &cli.global),
        Some(Commands::Regex(cmd)) => commands::regex_tool::run(cmd, &cli.global),
        Some(Commands::Case(args)) => commands::case::run(args, &cli.global),
        Some(Commands::Text(cmd)) => commands::text::run(cmd, &cli.global),
        Some(Commands::Time(cmd)) => commands::time_tool::run(cmd, &cli.global),
        Some(Commands::Qr(cmd)) => commands::qr::run(cmd, &cli.global),
        Some(Commands::Color(cmd)) => commands::color::run(cmd, &cli.global),
        Some(Commands::Diff(args)) => commands::diff::run(args, &cli.global),
        Some(Commands::Crypt(cmd)) => commands::crypt::run(cmd, &cli.global),
        Some(Commands::Net(cmd)) => commands::net::run(cmd, &cli.global),
        Some(Commands::Clip(cmd)) => commands::clip::run(cmd, &cli.global),
        Some(Commands::Sysinfo) => commands::sysinfo_cmd::run(&cli.global),
        Some(Commands::Completions(args)) => commands::completions::run(args, &cli.global),
    };

    if let Err(err) = &result
        && cli.global.verbose
    {
        for cause in err.chain().skip(1) {
            eprintln!("caused by: {cause}");
        }
    }

    result
}

fn is_top_level_help(args: &[std::ffi::OsString]) -> bool {
    let mut saw_command = false;
    for arg in args.iter().skip(1) {
        let Some(arg) = arg.to_str() else {
            continue;
        };
        match arg {
            "-h" | "--help" => return !saw_command,
            "--json" | "--no-color" | "--quiet" | "-q" | "--verbose" | "-v" | "--no-banner" => {}
            value if value.starts_with('-') => {}
            _ => saw_command = true,
        }
    }
    false
}

fn print_usage_hint() {
    println!("Usage: mpct [OPTIONS] <COMMAND>");
    println!();
    println!(
        "Commands: hash, checksum, encode, decode, jwt, uuid, ulid, nanoid, pass, totp, json, yaml, toml, regex, case, text, time, qr, color, diff, crypt, net, clip, sysinfo, completions, banner"
    );
    println!("Run `mpct --help` or `mpct <COMMAND> --help` for details.");
}
