mod r#macro;

use super::SyntaxError;
use crate::{ast::Primitive, *};
use analyzer::structure::{declaration, expression, repetition, typedef};
use ast::{BinaryOperator, Compound, Data, Datatype, Statement, UnaryOperator, Value};
use std::iter::Peekable;

// FUTURE: `syntax!` composition to define everything declaratively

syntax!(
  statement() with stream -> Statement<'a>:
    Token::Keyword("typedef") => Statement::Type(typedef(stream)?);
    Token::Keyword("return") => Statement::Return(expression(stream, vec![";"])?.0);
    Token::Keyword("int") => declaration(stream, Datatype::Type(Compound (Primitive::Int, 1)))?;
    Token::Keyword("float") => declaration(stream, Datatype::Type(Compound (Primitive::Float, 1)))?;
    Token::Keyword("short") => declaration(stream, Datatype::Type(Compound (Primitive::Short, 1)))?;
    Token::Keyword("long") => declaration(stream, Datatype::Type(Compound (Primitive::Long, 1)))?;
    Token::Keyword("char") => declaration(stream, Datatype::Type(Compound (Primitive::Byte, 1)))?;
    Token::Identifier(identifier) => declaration(stream, Datatype::Alias(identifier))?;
    Token::Keyword("for") => Statement::Loop(repetition(stream)?);
    Token::Symbol(";") => Statement::Noop;
);

syntax!(
  datatype() -> Datatype<'a>:
    Token::Keyword("int") => Datatype::Type(Compound (Primitive::Int, 1));
    Token::Keyword("float") => Datatype::Type(Compound (Primitive::Float, 1));
    Token::Keyword("short") => Datatype::Type(Compound (Primitive::Short, 1));
    Token::Keyword("long") => Datatype::Type(Compound (Primitive::Long, 1));
    Token::Keyword("char") => Datatype::Type(Compound (Primitive::Byte, 1));
    Token::Identifier(identifier) => Datatype::Alias(identifier);
);

syntax!(
  symbol(symbol: &str) -> Token<'a>:
    x if x == Token::Symbol(symbol) => x;
);

syntax!(
  identifier() -> &'a str:
    Token::Identifier(identifier) => identifier;
);

syntax!(
  index() -> usize:
    Token::Symbol("["), Token::Data(Literal::Integer(size), _), Token::Symbol("]") => size as usize;
);

syntax!(
  literal() -> Value<'a>:
  Token::Data(Literal::Character(x), _) => Value::Data(Data::Byte(x as i8));
  Token::Data(Literal::Integer(x), _) => Value::Data(Data::Integer(x));
  Token::Data(Literal::Floating(x), _) => Value::Data(Data::Float(x));
  Token::Data(Literal::Long(x), _) => Value::Data(Data::Long(x));
);

syntax!(
  unary_operator() -> UnaryOperator:
    Token::Symbol("-") => UnaryOperator::Negation;
    Token::Symbol("!") => UnaryOperator::Inversion;
);

syntax!(
  binary_operator() -> BinaryOperator:
    Token::Symbol("+") => BinaryOperator::Addition;
    Token::Symbol("-") => BinaryOperator::Subtraction;
    Token::Symbol("/") => BinaryOperator::Division;
    Token::Symbol("*") => BinaryOperator::Multiplication;
    Token::Symbol("%") => BinaryOperator::Remainder;
    Token::Symbol(">") => BinaryOperator::Greater;
    Token::Symbol("<") => BinaryOperator::Less;
    Token::Symbol(">=") => BinaryOperator::GreaterEqual;
    Token::Symbol("<=") => BinaryOperator::LessEqual;
    Token::Symbol("==") => BinaryOperator::Equal;
    Token::Symbol("!=") => BinaryOperator::NotEqual;
    Token::Symbol("&&") => BinaryOperator::And;
    Token::Symbol("||") => BinaryOperator::Or;
    Token::Symbol("&") => BinaryOperator::BitwiseAnd;
    Token::Symbol("|") => BinaryOperator::BitwiseOr;
    Token::Symbol("^") => BinaryOperator::BitwiseXor;
    Token::Symbol("<<") => BinaryOperator::LeftShift;
    Token::Symbol(">>") => BinaryOperator::RightShift;
);
