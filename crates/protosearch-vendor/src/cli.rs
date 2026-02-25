//! CLI to generate protos.
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_stdin::{FileOrStdin, FileOrStdout};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Compile {
        package: String,
        #[arg(default_value = "-")]
        input: FileOrStdin,
        #[arg(default_value = "-")]
        output: FileOrStdout,
        #[arg(short, long)]
        existing: Option<PathBuf>,
        #[arg(long, default_value_t = 100)]
        number_offset: u32,
    },
    Extract {
        #[arg(default_value = "-")]
        input: FileOrStdin,
        #[arg(default_value = "-")]
        output: FileOrStdout,
    },
    Render {
        #[arg(default_value = "-")]
        input: FileOrStdin,
        #[arg(default_value = "-")]
        output: FileOrStdout,
    },
}
