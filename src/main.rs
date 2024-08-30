use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Implementation of the lox programming language for code crafters
#[derive(Debug, Parser)]
#[clap(name = "jp-lox", version)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Tokenize the input file.
    Tokenize {
        /// Input lox file. 
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Tokenize { path } => {
            let file_contents = fs::read_to_string(&path)?;

            if !file_contents.is_empty() {
                unimplemented!("Tokenize the input file");
            } else {
                println!("EOF  null"); // Placeholder, remove this line when implementing the scanner
            }
        },
    }

    Ok(())
}
