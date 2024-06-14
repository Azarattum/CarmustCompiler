use super::{syntax::*, SyntaxError};
use crate::{ast::*, Token, TokenStream};
use std::iter::Peekable;

pub fn declaration<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    datatype: Datatype<'a>,
) -> Result<Statement<'a>, SyntaxError<'a>> {
    match (&datatype, symbol(stream, "=")) {
        (Datatype::Alias(identifier), Ok(_)) => {
            return Ok(Statement::Assignment(assignment(stream, identifier, ";")?))
        }
        _ => (),
    }

    let identifier = identifier(stream)?;
    Ok(match symbol(stream, "(") {
        Ok(_) => Statement::Function(function(stream, datatype, identifier)?),
        Err(_) => Statement::Variable(variable(stream, datatype, identifier)?),
    })
}

pub fn function<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    datatype: Datatype<'a>,
    name: &'a str,
) -> Result<Function<'a>, SyntaxError<'a>> {
    match symbol(stream, ")") {
        Ok(_) => (),
        Err(SyntaxError { expected, found }) => {
            return Err(SyntaxError {
                expected: format!("{} because function arguments are not supported", expected),
                found,
            })
        }
    }
    symbol(stream, "{")?;
    let body = block(stream, "}")?;

    Ok(Function {
        datatype,
        name,
        body,
    })
}

pub fn variable<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    datatype: Datatype<'a>,
    name: &'a str,
) -> Result<Variable<'a>, SyntaxError<'a>> {
    let assignment = match symbol(stream, "=") {
        Ok(_) => Some(Assignment {
            name,
            value: expression(stream, ";")?,
        }),
        Err(_) => {
            symbol(stream, ";")?;
            None
        }
    };

    Ok(Variable {
        datatype, // TODO: support arrays
        assignment,
        name,
    })
}

pub fn typedef<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Type<'a>, SyntaxError<'a>> {
    let datatype = datatype(stream)?;
    let name = identifier(stream)?;
    let size = index(stream).unwrap_or(0);

    let datatype = match (size > 1, datatype) {
        // FUTURE: support arrays of aliases
        (true, Datatype::Alias(_)) => {
            return Err(SyntaxError {
                expected: "no index because arrays of aliases are not supported".to_owned(),
                found: stream.peek().map(|&x| x),
            })
        }
        (true, Datatype::Type(Compound(primitive, _))) => Datatype::Type(Compound(primitive, size)),
        (false, datatype) => datatype,
    };

    symbol(stream, ";")?;

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

pub fn repetition<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Loop<'a>, SyntaxError<'a>> {
    // FUTURE: allow for arbitrary expressions
    symbol(stream, "(")?;
    let datatype = datatype(stream)?;
    let name = identifier(stream)?;
    let initialization = variable(stream, datatype, name)?;
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
