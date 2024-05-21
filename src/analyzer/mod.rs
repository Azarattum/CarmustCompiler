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

// fn index<'a>(stream: &mut Peekable<impl TokenStream<'a>>) -> Option<usize> {
//     match stream.peek() {
//         Some(Token::Symbol("[")) => match stream.skip(1).next() {
//             Some(Token::Data(Literal::Integer(size), _)) if size > 0 => match stream.next() {
//                 Some(Token::Symbol("]")) => Some(size as usize),
//                 _ => panic!("Expected ]!"),
//             },
//             _ => panic!("Expected index!"),
//         },
//         _ => None,
//     }
// }

fn typedef<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<(&'a str, DataType<'a>), SyntaxError<'a>> {
    let datatype = primitive(stream)?;
    let identifier = identifier(stream)?;
    let size = index(stream)?;
    semicolon(stream)?;

    // let datatype = match size {
    //     Some(size) => DataType::Array(datatype, size),
    //     None => DataType::Primitive(datatype),
    // };

    return Ok((identifier, DataType::Array(datatype, size)));
}

pub trait Analyzable<'a> {
    fn analyze(self) -> Result<Program<'a>, SyntaxError<'a>>;
}

impl<'a, T: TokenStream<'a>> Analyzable<'a> for T {
    fn analyze(self) -> Result<Program<'a>, SyntaxError<'a>> {
        program(&mut self.peekable())
    }
}
