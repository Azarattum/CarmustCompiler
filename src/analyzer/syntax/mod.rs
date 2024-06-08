mod r#macro;

use super::SyntaxError;
use crate::{ast::Primitive, *};
use ast::{BinaryOperator, UnaryOperator, Value};
use std::iter::Peekable;

syntax!(
  primitive() -> Primitive<'a>:
    Token::Keyword("int") => Primitive::Int;
    Token::Keyword("float") => Primitive::Float;
    Token::Identifier(identifier) => Primitive::Custom(identifier);
);

syntax!(
  keyword(word: &str) -> ():
    Token::Keyword(x) if x == word => ();
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
    Token::Data(Literal::Floating(x), _) => Value::Float(x);
    Token::Data(Literal::Integer(x), _) => Value::Integer(x);
    Token::Data(Literal::Character(x), _) => Value::Integer(x as i64);
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
