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
        types: HashMap::from([
            ("short", DataType::Primitive(Primitive::Short)),
            ("float", DataType::Primitive(Primitive::Float)),
            ("char", DataType::Primitive(Primitive::Char)),
            ("long", DataType::Primitive(Primitive::Long)),
            ("int", DataType::Primitive(Primitive::Int)),
        ]),
        functions: Vec::new(),
        globals: Vec::new(),
    };

    while let Some(token) = stream.peek().map(|&x| x) {
        match token {
            Token::Keyword("typedef") => {
                let (key, datatype) = typedef(stream)?;
                program.types.insert(key, datatype);
            }
            Token::Keyword(x) | Token::Identifier(x) if program.types.contains_key(x) => {
                program.globals.push(variable(stream)?);
            }
            unknown => panic!("Unexpected token {:?}!", unknown),
        }
    }

    return Ok(program);
}

fn variable<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<VariableDeclaration<'a>, SyntaxError<'a>> {
    let primitive = primitive(stream)?;
    let identifier = identifier(stream)?;
    symbol(stream, "=")?; // this is optional
    let value = expression(stream)?;
    symbol(stream, ";")?;

    Ok(VariableDeclaration {
        datatype: DataType::Primitive(primitive), // support arrays
        name: identifier,
        value: Some(value), // support just declaration
    })
}

fn typedef<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<(&'a str, DataType<'a>), SyntaxError<'a>> {
    keyword(stream, "typedef")?;
    let datatype = primitive(stream)?;
    let identifier = identifier(stream)?;
    let size = index(stream).unwrap_or(0);
    symbol(stream, ";")?;

    let datatype = match size {
        0 => DataType::Primitive(datatype),
        size => DataType::Array(datatype, size),
    };

    return Ok((identifier, datatype));
}

fn expression<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Expression<'a>, SyntaxError<'a>> {
    Ok(Expression::Literal(literal(stream)?))
}

pub trait Analyzable<'a> {
    fn analyze(self) -> Result<Program<'a>, SyntaxError<'a>>;
}

impl<'a, T: TokenStream<'a>> Analyzable<'a> for T {
    fn analyze(self) -> Result<Program<'a>, SyntaxError<'a>> {
        program(&mut self.peekable())
    }
}
