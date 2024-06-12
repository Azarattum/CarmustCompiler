#![feature(let_chains)]
#![feature(type_alias_impl_trait)]
#![feature(return_position_impl_trait_in_trait)]

mod analyzer;
mod error;
mod tokenizer;
mod translator;
mod types;

use analyzer::*;
use anyhow::Result;
use colored::Colorize;
use program::Program;
use std::fs::read_to_string;
use tokenizer::*;
use translator::*;
use types::*;

fn main() -> Result<()> {
    let path = "./assets/sample.c";
    let file = read_to_string(path)?;

    let tokens: Vec<Token> = file.tokenize().collect();
    println!("{}: {:?}\n", "Tokens".bold().cyan(), tokens);

    let ast = tokens.into_iter().analyze();
    let ast = ast.unwrap_or_else(|error| error.crash(&file, path));
    println!("{}: {:?}\n", "AST".bold().magenta(), ast);

    let mut program = Program::new();
    ast.translate(&mut program);
    println!("{}:\n{:?}\n", "IR".bold().green(), program);

    Ok(())
}
