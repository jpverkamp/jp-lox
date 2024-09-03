use std::io::Read;

use anyhow::Result;
use clap::{Parser as ClapParser, Subcommand};
use clap_stdin::FileOrStdin;
use env_logger;

mod builtins;
mod const_enum;
mod environment;
mod evaluator;
mod named_source;
mod parser;
mod span;
mod tokenizer;
mod values;

use environment::EnvironmentStack;
use evaluator::Evaluate;
use named_source::NamedSource;
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

    /// The input file (or - for stdin)
    #[arg(global=true)]
    input: Option<FileOrStdin>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Tokenize and print all tokens.
    Tokenize,
    /// Parse and print the AST.
    Parse,
    /// Evaluate the source expression.
    Evaluate,
    /// Run the source program.
    Run,
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

    let source = if args.input.is_none() {
        let name = "<stdin>".to_string();
        let mut contents = String::new();
        std::io::stdin().read_to_string(&mut contents)?;
        NamedSource::new(name, contents)
    } else {
        let input = args.input.unwrap();
        let name = if input.is_file() {
            input.filename().to_string()
        } else {
            "<stdin>".to_string()
        };
        let contents = input.contents()?;
        NamedSource::new(name, contents)
    };

    // ----- Tokenizing -----

    log::debug!("Tokenizing...");
    let mut tokenizer = Tokenizer::new(&source.bytes);

    if let Command::Tokenize = args.command {
        for token in &mut tokenizer {
            println!("{}", token.code_crafters_format());
        }

        if tokenizer.had_errors() {
            for error in tokenizer.iter_errors() {
                eprintln!("{}", error);
            }
            std::process::exit(65);
        }

        return Ok(());
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

    if parser.tokenizer_had_errors() {
        for error in parser.tokenizer_iter_errors() {
            eprintln!("{}", error);
        }
        std::process::exit(65);
    }

    if let Command::Parse = args.command {
        println!("{}", ast);
        return Ok(());
    }

    // ----- Evaluating -----

    match args.command {
        Command::Evaluate | Command::Run => {
            let mut env = EnvironmentStack::new();
            let output = match ast.evaluate(&mut env) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(70);
                }
            };

            // Eval prints the last command, run doesn't
            // For *reasons* numbers should't print .0 here
            if let Command::Evaluate = args.command {
                match output {
                    values::Value::Number(n) => println!("{n}"),
                    _ => println!("{}", output),
                }
            } else if let Command::Run = args.command {
                // Do nothing
            }
        }
        _ => {}
    }

    // Success (so far)
    Ok(())
}
