use anyhow::Result;
use clap::{Parser as ClapParser, Subcommand};
use clap_stdin::FileOrStdin;
use env_logger;

mod const_enum;
mod evaluator;
mod parser;
mod span;
mod tokenizer;
mod values;

use evaluator::Evaluate;
use parser::Parser;
use tokenizer::Tokenizer;

/// Implementation of the lox programming language for code crafters
#[derive(Debug, ClapParser)]
#[clap(name = "jp-lox", version)]
pub struct Args {
    /// Debug mode
    #[clap(short, long)]
    debug: bool,

    /// Subcommand to run
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
    /// Tokenize and parse the input file.
    Parse {
        /// Input lox file or - for stdin.
        input: FileOrStdin,
    },
    /// Tokenize, parse, and evaluate the input file.
    #[clap(aliases = &["run"])]
    Evaluate {
        /// Input lox file or - for stdin.
        input: FileOrStdin,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.debug {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::init();
    }

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
        }
        Command::Parse { input } => {
            let file_contents = input.contents()?;
            let tokenizer = Tokenizer::new(&file_contents);
            let mut parser = Parser::from(tokenizer);

            let ast = match parser.parse() {
                Ok(ast) => ast,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(65);
                }
            };

            if parser.encountered_tokenizer_error() {
                std::process::exit(65);
            }

            println!("{ast}");
        }
        Command::Evaluate { input } => {
            let file_contents = input.contents()?;
            let tokenizer = Tokenizer::new(&file_contents);
            let mut parser = Parser::from(tokenizer);

            let ast = match parser.parse() {
                Ok(ast) => ast,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(65);
                }
            };

            if parser.encountered_tokenizer_error() {
                std::process::exit(65);
            }

            let output = match ast.evaluate() {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(70);
                }
            };

            // And now, for reasons... we *don't* want 10.0, we want 10
            match output {
                values::Value::Nil => {},
                values::Value::Number(n) => println!("{}", n),
                _ => println!("{output}"),
            }
        }
    }

    Ok(())
}
