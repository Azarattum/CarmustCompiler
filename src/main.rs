#![feature(type_alias_impl_trait)]
#![feature(return_position_impl_trait_in_trait)]

mod analyzer;
mod error;
mod tokenizer;
mod types;

use analyzer::*;
use anyhow::Result;
use std::fs::read_to_string;
use tokenizer::*;
use types::*;

fn main() -> Result<()> {
    let path = "./assets/sample.c";
    let file = read_to_string(path)?;
    let tokens: Vec<Token> = file.tokenize().collect();
    println!("{:?}\n", tokens);

    let ast = tokens.into_iter().analyze();
    match ast {
        Ok(ast) => println!("{:?}", ast),
        Err(error) => println!("{}", error.error(&file, path)),
    }

    // let token_iter = file.tokenize();
    // let tokens: Vec<Token> = token_iter.take(3).collect();
    // println!("{:?}", ast);

    Ok(())
}
