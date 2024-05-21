use super::{syntax::*, SyntaxError};
use crate::{ast::*, TokenStream};
use std::iter::Peekable;

pub fn declaration<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Declaration<'a>, SyntaxError<'a>> {
    match keyword(stream, "typedef") {
        Ok(_) => Ok(Declaration::Type(typedef(stream)?)),
        Err(_) => {
            let primitive = primitive(stream)?;
            let identifier = identifier(stream)?;

            Ok(match symbol(stream, "(") {
                Ok(_) => Declaration::Function(function(stream, primitive, identifier)?),
                Err(_) => Declaration::Variable(variable(stream, primitive, identifier)?),
            })
        }
    }
}

fn function<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    primitive: Primitive<'a>,
    identifier: &'a str,
) -> Result<Function<'a>, SyntaxError<'a>> {
    // NOTE! Arguments are not supported in this implementation!
    symbol(stream, ")")?;
    symbol(stream, "{")?;
    // TODO: Parse body here
    symbol(stream, "}")?;

    Ok(Function {
        // NOTE! This implementation only supports primitive function return types
        datatype: DataType::Primitive(primitive),
        name: identifier,
        body: Vec::new(), // TODO: Make this an iterator
    })
}

fn variable<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    primitive: Primitive<'a>,
    identifier: &'a str,
) -> Result<Variable<'a>, SyntaxError<'a>> {
    symbol(stream, "=")?; // TODO: make this is optional
    let value = expression(stream)?;
    symbol(stream, ";")?;

    Ok(Variable {
        datatype: DataType::Primitive(primitive), // TODO: support arrays
        name: identifier,
        value: Some(value), // TODO: support declaration without alignment
    })
}

fn typedef<'a>(stream: &mut Peekable<impl TokenStream<'a>>) -> Result<Type<'a>, SyntaxError<'a>> {
    let datatype = primitive(stream)?;
    let name = identifier(stream)?;
    let size = index(stream).unwrap_or(0);
    symbol(stream, ";")?;

    let datatype = match size {
        0 => DataType::Primitive(datatype),
        size => DataType::Array(datatype, size),
    };

    return Ok(Type { name, datatype });
}

fn expression<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Expression<'a>, SyntaxError<'a>> {
    Ok(Expression::Literal(literal(stream)?))
    // TODO: support other expressions
}
