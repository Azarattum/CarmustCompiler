#![feature(let_chains)]
#![feature(type_alias_impl_trait)]
#![feature(return_position_impl_trait_in_trait)]

mod analyzer;
mod error;
mod tokenizer;
mod types;

use analyzer::*;
use anyhow::Result;
use colored::Colorize;
use std::fs::read_to_string;
use tokenizer::*;
use types::*;

fn main() -> Result<()> {
    let path = "./assets/sample.c";
    let file = read_to_string(path)?;
    let tokens: Vec<Token> = file.tokenize().collect();
    println!("{}: {:?}\n", "Tokens".bold().cyan(), tokens);

    let ast: Result<Vec<_>, _> = tokens.into_iter().analyze().collect();
    match ast {
        Ok(ast) => println!("{}: {:?}", "AST".bold().magenta(), ast),
        Err(error) => println!("{}", error.error(&file, path)),
    }

    Ok(())
}
