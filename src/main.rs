use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_stdin::FileOrStdin;

mod const_enum;

mod tokenizer;
use tokenizer::Tokenizer;

mod values;

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
        /// Input lox file or - for stdin. 
        input: FileOrStdin,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Tokenize { input } => {
            // Read from the file or stdin
            let file_contents = input.contents()?;

            let mut tokenizer = Tokenizer::new(&file_contents);
            for token in &mut tokenizer {
                println!("{}", token.code_crafters_format());
            }

            if tokenizer.encountered_error() {
                std::process::exit(65);
            }
        },
    }

    Ok(())
}
