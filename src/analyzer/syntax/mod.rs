mod r#macro;

use super::Primitive;
use super::SyntaxError;
use crate::*;
use std::iter::Peekable;

syntax!(
  primitive() -> Primitive<'a>:
    Token::Keyword("int") => Primitive::Int;
    Token::Keyword("float") => Primitive::Float;
    Token::Identifier(identifier) => Primitive::Custom(identifier);
);

syntax!(
    keyword(_word: &str) -> ():
      Token::Keyword(_word) => ();
);

syntax!(
  symbol(_symbol: &str) -> ():
    Token::Symbol(_symbol) => ();
);

syntax!(
  identifier() -> &'a str:
    Token::Identifier(identifier) => identifier;
);

syntax!(
  literal() -> i64:
    Token::Data(Literal::Integer(x), _) => x;
    // support other datatypes
);

syntax!(
  index() -> usize:
    Token::Symbol("["), Token::Data(Literal::Integer(size), _), Token::Symbol("]") => size as usize;
);
