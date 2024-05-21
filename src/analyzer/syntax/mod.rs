mod r#macro;

use super::SyntaxError;
use crate::{ast::Primitive, *};
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
  symbol(symbol: &str) -> ():
    Token::Symbol(x) if x == symbol => ();
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
  literal() -> i64:
    Token::Data(Literal::Integer(x), _) => x;
    // TODO: support other datatypes
);
