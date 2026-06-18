use anyhow::Result;
use chrono::{DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

use crate::{
    cli::{GlobalOptions, TimeCommand},
    error, output,
};

pub fn run(cmd: &TimeCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        TimeCommand::Now(args) => {
            let format = args.format.as_deref().unwrap_or("%+");
            let rendered = if args.utc {
                Utc::now().format(format).to_string()
            } else {
                Local::now().format(format).to_string()
            };
            output::write_or_json(
                global,
                || {
                    println!("{rendered}");
                    Ok(())
                },
                &serde_json::json!({ "time": rendered }),
            )
        }
        TimeCommand::Epoch(args) => {
            let dt = parse_datetime(&args.time)?;
            let epoch = dt.timestamp();
            output::write_or_json(
                global,
                || {
                    println!("{epoch}");
                    Ok(())
                },
                &serde_json::json!({ "epoch": epoch }),
            )
        }
        TimeCommand::Iso(args) => {
            let dt = Utc
                .timestamp_opt(args.epoch, 0)
                .single()
                .ok_or_else(|| error::msg("epoch value is outside the supported range"))?;
            let rendered = dt.to_rfc3339();
            output::write_or_json(
                global,
                || {
                    println!("{rendered}");
                    Ok(())
                },
                &serde_json::json!({ "iso": rendered }),
            )
        }
        TimeCommand::Tz(args) => {
            let from: Tz = args
                .from
                .parse()
                .map_err(|_| error::msg(format!("unknown source timezone `{}`", args.from)))?;
            let to: Tz = args
                .to
                .parse()
                .map_err(|_| error::msg(format!("unknown target timezone `{}`", args.to)))?;
            let naive = parse_naive_datetime(&args.time)?;
            let source = match from.from_local_datetime(&naive) {
                LocalResult::Single(dt) => dt,
                LocalResult::Ambiguous(a, _) => a,
                LocalResult::None => {
                    return Err(error::msg("source time does not exist in that timezone"));
                }
            };
            let converted = source.with_timezone(&to).to_rfc3339();
            output::write_or_json(
                global,
                || {
                    println!("{converted}");
                    Ok(())
                },
                &serde_json::json!({ "time": converted }),
            )
        }
    }
}

fn parse_datetime(input: &str) -> Result<DateTime<Utc>> {
    if input == "now" {
        return Ok(Utc::now());
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt.with_timezone(&Utc));
    }
    let naive = parse_naive_datetime(input)?;
    match Local.from_local_datetime(&naive) {
        LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
        LocalResult::Ambiguous(a, _) => Ok(a.with_timezone(&Utc)),
        LocalResult::None => Err(error::msg(
            "local time does not exist in the local timezone",
        )),
    }
}

fn parse_naive_datetime(input: &str) -> Result<NaiveDateTime> {
    for format in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S"] {
        if let Ok(dt) = NaiveDateTime::parse_from_str(input, format) {
            return Ok(dt);
        }
    }
    if let Ok(date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| error::msg("date is outside supported range"));
    }
    Err(error::msg(
        "time must be RFC3339, now, YYYY-MM-DD, or YYYY-MM-DD HH:MM:SS",
    ))
}
