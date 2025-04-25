use super::{
    syntax::{identifier, index},
    SyntaxError,
};
use crate::{
    analyzer::syntax::{binary_operator, literal, symbol, unary_operator},
    ast::{Expression, Operator, Value},
    Token, TokenStream,
};
use std::iter::Peekable;

impl<'a> Expression<'a> {
    pub fn from_stream(
        stream: &mut Peekable<impl TokenStream<'a>>,
        terminators: Vec<&str>,
    ) -> Result<(Self, Token<'a>), SyntaxError<'a>> {
        let expression_error = SyntaxError {
            expected: "expression".to_owned(),
            found: stream.peek().map(|&x| x),
        };

        let mut output = Vec::new();
        let mut stack = Vec::new();
        let mut complete = false;

        let mut apply = |mutation: Mutation<'a>| -> Result<(), SyntaxError<'a>> {
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

        let terminator = loop {
            match term(stream, &mut stack, complete) {
                Ok((mutations, completed)) => {
                    complete = completed;
                    for mutation in mutations {
                        apply(mutation)?;
                    }
                }
                Err(error)
                    if terminators
                        .iter()
                        .any(|x| Some(Token::Symbol(x)) == error.found) =>
                {
                    break stream.next().unwrap();
                }
                Err(error) => return Err(error),
            }
        };

        while let Some(&Operator::Binary(_) | &Operator::Unary(_)) = stack.last() {
            apply(Mutation::Operator(stack.pop()))?;
        }

        match (stack.len(), output.len()) {
            (0, 1) => Ok((output.pop().unwrap(), terminator)),
            _ => Err(expression_error),
        }
    }
}

fn term<'a>(
    stream: &mut Peekable<impl TokenStream<'a>>,
    stack: &mut Vec<Operator>,
    complete: bool,
) -> Result<(Vec<Mutation<'a>>, bool), SyntaxError<'a>> {
    let mut mutations = Vec::new();
    let mut completed = false;

    literal(stream)
        .and_then(|x| {
            completed = true;
            mutations.push(Mutation::Expression(Expression::Value(x)));
            Ok(())
        })
        .or_else(|_: SyntaxError<'a>| {
            let identifier = identifier(stream)?;
            completed = true;

            let index = index(stream).unwrap_or(0);
            let value = Expression::Value(Value::Pointer(identifier, index));

            mutations.push(Mutation::Expression(value));
            Ok(())
        })
        .or_else(|_: SyntaxError<'a>| {
            if !complete && let Ok(op) = unary_operator(stream) {
                stack.push(Operator::Unary(op));
                return Ok(());
            }

            let op = binary_operator(stream)?;
            let mut top = stack.last();
            completed = false;

            while let Some(&operator) = top
                && match operator {
                    Operator::Binary(x) if x.precedence() <= op.precedence() => true,
                    Operator::Unary(_) => true,
                    _ => false,
                }
            {
                mutations.push(Mutation::Operator(stack.pop()));
                top = stack.last();
            }

            stack.push(Operator::Binary(op));
            Ok(())
        })
        .or_else(|_: SyntaxError<'a>| {
            symbol(stream, "(")?;
            stack.push(Operator::Group);
            completed = false;
            Ok(())
        })
        .or_else(|_: SyntaxError<'a>| {
            let token = stream.peek().map(|&x| x);
            if token == Some(Token::Symbol(")")) && !stack.contains(&Operator::Group) {
                return Err(SyntaxError {
                    expected: "expression term".to_owned(),
                    found: token,
                });
            }

            symbol(stream, ")")?;
            let mut top = stack.last();
            completed = true;

            while let Some(&Operator::Binary(_) | &Operator::Unary(_)) = top {
                mutations.push(Mutation::Operator(stack.pop()));
                top = stack.last();
            }

            stack.pop();
            Ok(())
        })
        .or_else(|error: SyntaxError<'a>| {
            Err(SyntaxError {
                expected: "expression term".to_owned(),
                found: error.found,
            })
        })?;

    Ok((mutations, completed))
}

enum Mutation<'a> {
    Expression(Expression<'a>),
    Operator(Option<Operator>),
}
