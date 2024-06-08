use super::{syntax::*, SyntaxError};
use crate::{ast::*, TokenStream};
use std::iter::Peekable;

pub fn declaration<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    primitive: Primitive<'a>,
) -> Result<Statement<'a>, SyntaxError<'a>> {
    match (&primitive, symbol(stream, "=")) {
        (Primitive::Custom(identifier), Ok(_)) => {
            return Ok(Statement::Assignment(assignment(stream, identifier)?))
        }
        _ => (),
    }

    let identifier = identifier(stream)?;
    Ok(match symbol(stream, "(") {
        Ok(_) => Statement::Function(function(stream, primitive, identifier)?),
        Err(_) => Statement::Variable(variable(stream, primitive, identifier)?),
    })
}

pub fn function<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    primitive: Primitive<'a>,
    identifier: &'a str,
) -> Result<Function<'a>, SyntaxError<'a>> {
    // NOTE! Arguments are not supported in this implementation!
    // TODO: add hint support to SyntaxError?
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

pub fn variable<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    primitive: Primitive<'a>,
    identifier: &'a str,
) -> Result<Variable<'a>, SyntaxError<'a>> {
    let value = match symbol(stream, "=") {
        Ok(_) => Some(expression(stream)?),
        Err(_) => None,
    };

    Ok(Variable {
        datatype: DataType::Primitive(primitive), // TODO: support arrays
        name: identifier,
        value, // TODO: support declaration without alignment
    })
}

pub fn typedef<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Type<'a>, SyntaxError<'a>> {
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

pub fn expression<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Expression<'a>, SyntaxError<'a>> {
    Expression::from_stream(stream)
}

pub fn assignment<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    identifier: &'a str,
) -> Result<Assignment<'a>, SyntaxError<'a>> {
    Ok(Assignment {
        name: identifier,
        value: expression(stream)?,
    })
}
