use std::fs;

use anyhow::{Context, Result};
use similar::{ChangeTag, TextDiff};

use crate::{
    cli::{DiffArgs, GlobalOptions},
    output,
};

pub fn run(args: &DiffArgs, global: &GlobalOptions) -> Result<()> {
    let old = fs::read_to_string(&args.file_a)
        .with_context(|| format!("failed to read {}", args.file_a.display()))?;
    let new = fs::read_to_string(&args.file_b)
        .with_context(|| format!("failed to read {}", args.file_b.display()))?;
    let diff = TextDiff::from_lines(&old, &new);
    let text = if args.side_by_side {
        side_by_side(&diff)
    } else {
        diff.unified_diff()
            .header(
                &args.file_a.display().to_string(),
                &args.file_b.display().to_string(),
            )
            .to_string()
    };
    output::write_or_json(
        global,
        || {
            print!("{text}");
            Ok(())
        },
        &serde_json::json!({ "diff": text }),
    )
}

fn side_by_side(diff: &TextDiff<'_, '_, str>) -> String {
    let mut out = String::new();
    for change in diff.iter_all_changes() {
        let marker = match change.tag() {
            ChangeTag::Delete => '<',
            ChangeTag::Insert => '>',
            ChangeTag::Equal => ' ',
        };
        out.push(marker);
        out.push(' ');
        out.push_str(change.value());
    }
    out
}
