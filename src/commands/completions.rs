use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::cli::{Cli, CompletionArgs, CompletionShell, GlobalOptions};

pub fn run(args: &CompletionArgs, _global: &GlobalOptions) -> Result<()> {
    let shell = match args.shell {
        CompletionShell::Bash => Shell::Bash,
        CompletionShell::Zsh => Shell::Zsh,
        CompletionShell::Fish => Shell::Fish,
        CompletionShell::Powershell => Shell::PowerShell,
        CompletionShell::Elvish => Shell::Elvish,
    };
    let mut command = Cli::command();
    generate(shell, &mut command, "mpct", &mut std::io::stdout());
    Ok(())
}
