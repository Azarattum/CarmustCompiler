mod r#macro;

use super::Primitive;
use super::SyntaxError;
use crate::*;
use std::iter::Peekable;

syntax!(
  primitive -> Primitive<'a>:
    Token::Keyword("int") => Primitive::Int;
    Token::Keyword("float") => Primitive::Float;
    Token::Identifier(identifier) => Primitive::Custom(identifier);
);

syntax!(
  semicolon -> ():
    Token::Symbol(";") => ();
);

syntax!(
  identifier -> &'a str:
    Token::Identifier(identifier) => identifier;
);

syntax!(
  index -> usize:
    Token::Symbol("["), Token::Data(Literal::Integer(size), _), Token::Symbol("]") => size as usize;
);
