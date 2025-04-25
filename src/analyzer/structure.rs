use super::{syntax::*, SyntaxError};
use crate::{ast::*, Token, TokenStream};
use std::iter::Peekable;

pub fn declaration<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    mut datatype: Datatype<'a>,
) -> Result<Statement<'a>, SyntaxError<'a>> {
    match (&datatype, stream.peek()) {
        (Datatype::Alias(identifier), Some(Token::Symbol("="))) => {
            symbol(stream, "=")?;
            return Ok(Statement::Assignment(assignment(
                stream,
                (identifier, 0),
                ";",
            )?));
        }
        (Datatype::Alias(identifier), Some(Token::Symbol("["))) => {
            let index = index(stream)?;
            symbol(stream, "=")?;
            return Ok(Statement::Assignment(assignment(
                stream,
                (identifier, index),
                ";",
            )?));
        }
        _ => {}
    }

    let (identifier, size) = identifier(stream)?;
    if size != 0 {
        match datatype {
            Datatype::Type(value) => {
                datatype = Datatype::Type(Compound(value.0, size));
            }
            _ => {
                // FUTURE: support arrays of aliases
                return Err(SyntaxError {
                    expected: "no index because arrays of aliases are not supported".to_owned(),
                    found: Some(Token::Identifier(identifier)),
                });
            }
        }
    }

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
        Ok(_) => Some(assignment(stream, (name, 0), ";")?),
        Err(_) => {
            symbol(stream, ";")?;
            None
        }
    };

    Ok(Variable {
        datatype,
        assignment,
        name,
    })
}

pub fn typedef<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Type<'a>, SyntaxError<'a>> {
    let datatype = datatype(stream)?;
    let (name, size) = identifier(stream)?;

    let datatype = match (size > 1, datatype) {
        // FUTURE: support arrays of aliases
        (true, Datatype::Alias(_)) => {
            return Err(SyntaxError {
                expected: "no index because arrays of aliases are not supported".to_owned(),
                found: Some(Token::Identifier(name)),
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
    terminators: Vec<&str>,
) -> Result<(Expression<'a>, Token<'a>), SyntaxError<'a>> {
    Expression::from_stream(stream, terminators)
}

pub fn assignment<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    identifier: (&'a str, usize),
    terminator: &str,
) -> Result<Assignment<'a>, SyntaxError<'a>> {
    if let Some(Token::Symbol("{")) = stream.peek() {
        Ok(Assignment {
            identifier,
            value: Initializer::List(initializer(stream, terminator)?),
        })
    } else {
        Ok(Assignment {
            identifier,
            value: Initializer::Expression(expression(stream, vec![terminator])?.0),
        })
    }
}

pub fn repetition<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Loop<'a>, SyntaxError<'a>> {
    symbol(stream, "(")?;

    let datatype = datatype(stream)?;
    let (name, _) = identifier(stream)?;
    let initialization = variable(stream, datatype, name)?;

    let condition = expression(stream, vec![";"])?.0;

    let identifier = identifier(stream)?;
    symbol(stream, "=")?;
    let increment = assignment(stream, identifier, ")")?;

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
            Err(error) if error.found == expected && error.expected == "statement" => {
                break;
            }
            Err(error) => return Err(error),
        }
    }

    if terminator != "" {
        symbol(stream, terminator)?;
    }

    return Ok(block);
}

fn initializer<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    terminator: &str,
) -> Result<Vec<Expression<'a>>, SyntaxError<'a>> {
    symbol(stream, "{")?;
    let mut elements = Vec::new();
    loop {
        let (expression, terminator) = expression(stream, vec![",", "}"])?;
        elements.push(expression);

        if terminator == Token::Symbol("}") {
            break;
        }
    }
    symbol(stream, terminator)?;
    Ok(elements)
}
