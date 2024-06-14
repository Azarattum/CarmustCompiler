mod r#macro;

use super::SyntaxError;
use crate::{ast::Primitive, *};
use analyzer::structure::{declaration, expression, repetition, typedef};
use ast::{BinaryOperator, Data, Statement, UnaryOperator, Value};
use std::iter::Peekable;

// FUTURE: `syntax!` composition to define everything declaratively

syntax!(
  statement() with stream -> Statement<'a>:
    Token::Keyword("typedef") => Statement::Type(typedef(stream)?);
    Token::Keyword("return") => Statement::Return(expression(stream, ";")?);
    Token::Keyword("int") => declaration(stream, Primitive::Int)?;
    Token::Keyword("float") => declaration(stream, Primitive::Float)?;
    Token::Keyword("short") => declaration(stream, Primitive::Short)?;
    Token::Keyword("long") => declaration(stream, Primitive::Long)?;
    Token::Keyword("char") => declaration(stream, Primitive::Byte)?;
    Token::Identifier(identifier) => declaration(stream, Primitive::Custom(identifier))?;
    Token::Keyword("for") => Statement::Loop(repetition(stream)?);
);

syntax!(
  primitive() -> Primitive<'a>:
    Token::Keyword("int") => Primitive::Int;
    Token::Keyword("float") => Primitive::Float;
    Token::Keyword("short") => Primitive::Short;
    Token::Keyword("long") => Primitive::Long;
    Token::Keyword("char") => Primitive::Byte;
    Token::Identifier(identifier) => Primitive::Custom(identifier);
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
    Token::Symbol("["), Token::Data(Literal::Integer(size), _) if size > 0, Token::Symbol("]") => size as usize;
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
