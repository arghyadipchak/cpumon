use std::{fs, path::Path};

use clap::CommandFactory;
use clap_complete::{Shell, generate_to};

#[allow(dead_code, clippy::all, clippy::pedantic)]
mod utils {
    use crate::cpumon_build_cli::CliError;

    pub fn get_cpu_count() -> Result<usize, CliError> {
        Ok(1)
    }
}

#[allow(dead_code, clippy::all, clippy::pedantic)]
#[path = "src/cli.rs"]
mod cpumon_build_cli;

fn main() {
    println!("cargo:rerun-if-changed=src/cli.rs");

    let out_dir = Path::new("completions");
    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!("cargo:warning=Failed to create completions directory: {e}");
        return;
    }

    let mut cmd = cpumon_build_cli::Cli::command();
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        if let Err(e) = generate_to(shell, &mut cmd, "cpumon", out_dir) {
            eprintln!("cargo:warning=Failed to generate completions for {shell:?}: {e}");
        }
    }
}
