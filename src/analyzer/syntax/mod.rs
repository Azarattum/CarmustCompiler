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
  // TODO: support `if size > 0`
  // TODO: support peaking
    Token::Symbol("["), Token::Data(Literal::Integer(size), _), Token::Symbol("]") => size as usize;
);

// pub fn identifier<'a>(
//     stream: &mut Peekable<impl TokenStream<'a>>,
// ) -> Result<&'a str, SyntaxError<'a>> {
//     match stream.next() {
//         Some(Token::Identifier(identifier)) => match stream.next() {
//             Some(Token::Symbol("[")) => Ok(identifier),
//             token => Err(SyntaxError {
//                 expected: stringify!(identifier),
//                 found: token,
//             }),
//         },
//         token => Err(SyntaxError {
//             expected: stringify!(identifier),
//             found: token,
//         }),
//     }
// }

// fn index<'a>(stream: &mut Peekable<impl TokenStream<'a>>) -> Option<usize> {
//     match stream.peek() {
//         Some(Token::Symbol("[")) => match stream.skip(1).next() {
//             Some(Token::Data(Literal::Integer(size), _)) if size > 0 => match stream.next() {
//                 Some(Token::Symbol("]")) => Some(size as usize),
//                 _ => panic!("Expected ]!"),
//             },
//             _ => panic!("Expected index!"),
//         },
//         _ => None,
//     }
// }
