use anyhow::{Context, Result};

use crate::{
    cli::{ClipCommand, GlobalOptions},
    error, output,
};

pub fn run(cmd: &ClipCommand, global: &GlobalOptions) -> Result<()> {
    match cmd {
        ClipCommand::Get => {
            let mut clipboard = clipboard()?;
            let text = clipboard
                .get_text()
                .map_err(|err| error::msg(format!("failed to read clipboard text: {err}")))?;
            output::write_or_json(
                global,
                || {
                    println!("{text}");
                    Ok(())
                },
                &serde_json::json!({ "text": text }),
            )
        }
        ClipCommand::Set(args) => {
            let text = output::read_input_string(&args.text, &None)?;
            let mut clipboard = clipboard()?;
            clipboard
                .set_text(text)
                .map_err(|err| error::msg(format!("failed to set clipboard text: {err}")))?;
            if !global.quiet {
                println!("OK");
            }
            Ok(())
        }
    }
}

fn clipboard() -> Result<arboard::Clipboard> {
    arboard::Clipboard::new().with_context(|| {
        "failed to access the system clipboard; on headless Linux, start a graphical session or set up a clipboard provider"
    })
}
