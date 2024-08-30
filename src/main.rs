use anyhow::Result;
use clap::{Parser as ClapParser, Subcommand};
use clap_stdin::FileOrStdin;

mod const_enum;
mod parser;
mod tokenizer;
mod values;

use parser::Parser;
use tokenizer::Tokenizer;

/// Implementation of the lox programming language for code crafters
#[derive(Debug, ClapParser)]
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
    /// Parse the input file.
    Parse {
        /// Input lox file or - for stdin. 
        input: FileOrStdin,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Tokenize { input } => {
            let file_contents = input.contents()?;
            let mut tokenizer = Tokenizer::new(&file_contents);

            for token in &mut tokenizer {
                println!("{}", token.code_crafters_format());
            }

            if tokenizer.encountered_error() {
                std::process::exit(65);
            }
        },
        Command::Parse { input } => {
            let file_contents = input.contents()?;
            let tokenizer = Tokenizer::new(&file_contents);
            let mut parser = Parser::from(tokenizer);

            let ast = parser.parse()?;
            
            println!("{ast}");
        },
    }

    Ok(())
}
