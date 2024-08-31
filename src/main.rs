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
    /// Print the last command's output (including nil)
    Evaluate {
        /// Input lox file or - for stdin.
        input: FileOrStdin,
    },
    /// Run the input file, do not print the last command's output.
    Run {
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

    // ----- Shared filename / contents loading -----

    let (file_name, file_contents) = match args.command {
        Command::Tokenize { ref input }
        | Command::Parse { ref input }
        | Command::Evaluate { ref input }
        | Command::Run { ref input } => (
            if input.is_file() {
                input.filename().to_string()
            } else {
                "stdin".to_string()
            },
            input.clone().contents()?
        )
    };

    // ----- Tokenizing -----

    log::debug!("Tokenizing {}", file_name);
    let mut tokenizer = Tokenizer::new(&file_contents);

    if let Command::Tokenize { .. } = args.command {
        for token in &mut tokenizer {
            println!("{}", token.code_crafters_format());
        }

        if tokenizer.encountered_error() {
            std::process::exit(65);
        } else {
            return Ok(());
        }
    }

    // ----- Parsing -----

    log::debug!("Parsing...");
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

    if let Command::Parse { .. } = args.command {
        println!("{}", ast);
        return Ok(());
    }

    // ----- Evaluating -----

    let output = match ast.evaluate() {
        Ok(value) => value,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(70);
        }
    };

    // Eval prints the last command, run doesn't
    // For *reasons* numbers should't print .0 here
    if let Command::Evaluate { .. } = args.command {
        match output {
            values::Value::Number(n) => println!("{n}"),
            _ => println!("{}", output),
        }
    } else if let Command::Run { .. } = args.command {
        // Do nothing
    }

    // Success (so far)
    Ok(())
}
