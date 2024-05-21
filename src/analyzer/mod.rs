mod ast;
mod syntax;

use self::ast::*;
use self::syntax::*;
use crate::error::*;
use crate::{Token, TokenStream};
use std::{collections::HashMap, iter::Peekable};

fn program<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Program<'a>, SyntaxError<'a>> {
    let mut program = Program {
        typedefs: HashMap::new(),
        functions: Vec::new(),
        globals: Vec::new(),
    };

    while let Some(token) = stream.next() {
        match token {
            Token::Keyword("typedef") => {
                let (k, v) = typedef(stream)?;
                program.typedefs.insert(k, v);
            }
            unknown => panic!("Unexpected token {:?}!", unknown),
        }
    }

    return Ok(program);
}

fn typedef<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<(&'a str, DataType<'a>), SyntaxError<'a>> {
    let datatype = primitive(stream)?;
    let identifier = identifier(stream)?;
    let size = index(stream).unwrap_or(0);
    semicolon(stream)?;

    let datatype = match size {
        0 => DataType::Primitive(datatype),
        size => DataType::Array(datatype, size),
    };

    return Ok((identifier, datatype));
}

pub trait Analyzable<'a> {
    fn analyze(self) -> Result<Program<'a>, SyntaxError<'a>>;
}

impl<'a, T: TokenStream<'a>> Analyzable<'a> for T {
    fn analyze(self) -> Result<Program<'a>, SyntaxError<'a>> {
        program(&mut self.peekable())
    }
}
