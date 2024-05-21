#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token<'a> {
    Data(Literal, &'a str),
    Identifier(&'a str),
    Keyword(&'a str),
    Symbol(&'a str),
    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Literal {
    Integer(i64),
    Floating(f64),
    Character(char),
}

pub trait TokenStream<'a>: Iterator<Item = Token<'a>> {}

impl<'a, T: Iterator<Item = Token<'a>>> TokenStream<'a> for T {}
