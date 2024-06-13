#![feature(let_chains)]
#![feature(iter_intersperse)]
#![feature(type_alias_impl_trait)]
#![feature(return_position_impl_trait_in_trait)]

mod analyzer;
mod assembly;
mod error;
mod executor;
mod tokenizer;
mod translator;
mod types;

use analyzer::*;
use assembly::*;
use colored::Colorize;
use error::*;
use executor::*;
use program::Program;
use std::fs::read_to_string;
use tokenizer::*;
use translator::*;
use types::*;

fn main() {
    let path = "./assets/sample.c";
    let file = read_to_string(path).expect(&format!("File at {path} does not exist!"));

    let tokens: Vec<Token> = file.tokenize().collect();
    println!("{}: {:?}\n", "Tokens".bold().cyan(), tokens);

    let ast = tokens.into_iter().analyze();
    let ast = ast.unwrap_or_else(|error| error.crash(&file, path));
    println!("{}: {:?}\n", "AST".bold().magenta(), ast);

    let mut program = Program::new();
    ast.translate(&mut program)
        .unwrap_or_else(|error| error.crash(&file, path));
    println!("{}:\n{:?}\n", "IR".bold().green(), program);

    let assembly = program
        .assemble()
        .unwrap_or_else(|error| error.crash(&file, path));
    println!("{}:\n{}\n", "ASM".bold().yellow(), assembly);

    let result = assembly
        .execute(path)
        .unwrap_or_else(|error| error.crash(&file, path));
    println!("{}: {}\n", "Execution Result".bold().blue(), result);
}
