use super::{
    syntax::{identifier, index},
    SyntaxError,
};
use crate::{
    analyzer::syntax::{binary_operator, literal, symbol, unary_operator},
    ast::{Expression, Operator, Value},
    TokenStream,
};
use std::iter::Peekable;

impl<'a> Expression<'a> {
    pub fn from_stream(
        stream: &mut Peekable<impl TokenStream<'a>>,
    ) -> Result<Self, SyntaxError<'a>> {
        expression(stream)
    }
}

fn expression<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
) -> Result<Expression<'a>, SyntaxError<'a>> {
    let expression_error = SyntaxError {
        expected: "expression".to_owned(),
        found: stream.peek().map(|&x| x),
    };

    let mut stack = Vec::new();
    let mut output = Vec::new();
    let mut mid_operation = true;

    enum Mutation<'a> {
        Expression(Expression<'a>),
        Operator(Option<Operator>),
    }

    let mut save = |mutation: Mutation<'a>| -> Result<(), SyntaxError<'a>> {
        let expression = match mutation {
            Mutation::Expression(expression) => Some(expression),
            Mutation::Operator(Some(operator)) => match operator {
                Operator::Group => None,
                Operator::Unary(op) => match output.pop() {
                    Some(lhs) => Some(Expression::Unary {
                        lhs: Box::new(lhs),
                        op,
                    }),
                    None => return Err(expression_error.clone()),
                },
                Operator::Binary(op) => match (output.pop(), output.pop()) {
                    (Some(rhs), Some(lhs)) => Some(Expression::Binary {
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        op,
                    }),
                    _ => return Err(expression_error.clone()),
                },
            },
            _ => None,
        };

        if let Some(expression) = expression {
            output.push(expression);
        }

        Ok(())
    };

    loop {
        match symbol(stream, ";") {
            Err(_) => literal(stream)
                .and_then(|x| {
                    mid_operation = false;
                    save(Mutation::Expression(Expression::Value(x)))?;
                    Ok(())
                })
                .or_else(|_: SyntaxError<'a>| {
                    let identifier = identifier(stream)?;
                    mid_operation = false;

                    let index = index(stream);
                    dbg!(&index);
                    let value = Expression::Value(match index {
                        Ok(index) => Value::Array(identifier, index),
                        Err(_) => Value::Identifier(identifier),
                    });

                    save(Mutation::Expression(value))?;
                    Ok(())
                })
                .or_else(|_: SyntaxError<'a>| {
                    if mid_operation && let Ok(op) = unary_operator(stream) {
                        mid_operation = false;
                        stack.push(Operator::Unary(op));
                        return Ok(())
                    }

                    let op = binary_operator(stream)?;
                    mid_operation = true;
                    let mut top = stack.last();

                    while let Some(&operator) = top && match operator {
                        Operator::Binary(x) if x.precedence() >= op.precedence() => true,
                        Operator::Unary(_) => true,
                        _ => false,
                    } {
                        save(Mutation::Operator(stack.pop()))?;
                        top = stack.last();
                    }

                    stack.push(Operator::Binary(op));
                    Ok(())
                })
                .or_else(|_: SyntaxError<'a>| {
                    symbol(stream, "(")?;
                    mid_operation = true;
                    stack.push(Operator::Group);
                    Ok(())
                })
                .or_else(|_: SyntaxError<'a>| {
                    symbol(stream, ")")?;
                    mid_operation = false;
                    let mut top = stack.last();

                    while let Some(&Operator::Binary(_) | &Operator::Unary(_)) = top {
                        save(Mutation::Operator(stack.pop()))?;
                        top = stack.last();
                    }

                    match top {
                        Some(&Operator::Group) => {
                            stack.pop();
                            Ok(())
                        }
                        _ => Err(expression_error.clone()),
                    }
                })
                .or_else(|error: SyntaxError<'a>| {
                    Err(SyntaxError {
                        expected: "expression term".to_owned(),
                        found: error.found,
                    })
                })?,
            Ok(_) => break,
        }
    }

    while stack.len() > 0 {
        save(Mutation::Operator(stack.pop()))?;
    }

    match output.len() {
        1 => Ok(output.pop().unwrap()),
        _ => Err(expression_error),
    }
}
