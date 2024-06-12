use super::{syntax::*, SyntaxError};
use crate::{ast::*, Token, TokenStream};
use std::iter::{self, Peekable};

pub fn declaration<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    primitive: Primitive<'a>,
) -> Result<Statement<'a>, SyntaxError<'a>> {
    match (&primitive, symbol(stream, "=")) {
        (Primitive::Custom(identifier), Ok(_)) => {
            return Ok(Statement::Assignment(assignment(stream, identifier, ";")?))
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
    let body = block(stream, "}")?;

    Ok(Function {
        // NOTE! This implementation only supports primitive function return types
        datatype: DataType::Primitive(primitive),
        name: identifier,
        body: body,
    })
}

pub fn variable<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    primitive: Primitive<'a>,
    identifier: &'a str,
) -> Result<Variable<'a>, SyntaxError<'a>> {
    let value = match symbol(stream, "=") {
        Ok(_) => Some(expression(stream, ";")?),
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
    terminator: &str,
) -> Result<Expression<'a>, SyntaxError<'a>> {
    Expression::from_stream(stream, terminator)
}

pub fn assignment<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    identifier: &'a str,
    terminator: &str,
) -> Result<Assignment<'a>, SyntaxError<'a>> {
    Ok(Assignment {
        name: identifier,
        value: expression(stream, terminator)?,
    })
}

// TODO: allow for arbitrary expressions?
pub fn repetition<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Loop<'a>, SyntaxError<'a>> {
    symbol(stream, "(")?;
    let primitive = primitive(stream)?;
    let name = identifier(stream)?;
    let initialization = variable(stream, primitive, name)?;
    let condition = expression(stream, ";")?;
    let name = identifier(stream)?;
    symbol(stream, "=")?;
    let increment = assignment(stream, name, ")")?;
    symbol(stream, "{")?;
    let body = block(stream, "}")?;

    Ok(Loop {
        initialization,
        condition,
        increment,
        body,
    })
}

pub fn block<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    terminator: &str,
) -> Result<Vec<Statement<'a>>, SyntaxError<'a>> {
    let expected = match terminator {
        "" => None,
        x => Some(Token::Symbol(x)),
    };

    let mut block = Vec::new();
    loop {
        match statement(stream) {
            Ok(decl) => block.push(decl),
            Err(error) if error.found == expected => break,
            Err(error) => return Err(error),
        }
    }

    if terminator != "" {
        symbol(stream, terminator)?;
    }

    return Ok(block);
}
