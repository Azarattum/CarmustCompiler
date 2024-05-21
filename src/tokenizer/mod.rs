mod known;

use self::known::*;
use crate::types::*;

fn to_token<'a>(text: &'a str) -> Token {
    match text {
        x if KEYWORDS.contains(&x) => Token::Keyword(x),
        x if SYMBOLS.contains(&x) => Token::Symbol(x),
        x if x.parse::<i64>().is_ok() => Token::Data(Literal::Integer(x.parse().unwrap()), x),
        x if x.parse::<f64>().is_ok() => Token::Data(Literal::Floating(x.parse().unwrap()), x),
        x if x.len() == 3 && x.trim_matches('\'').len() == 1 => {
            Token::Data(Literal::Character(x.chars().nth(1).unwrap()), x)
        }
        x if x.starts_with("//") => Token::Comment,
        x if x.starts_with("/*") => Token::Comment,
        x => Token::Identifier(x),
    }
}

pub trait Tokenizable {
    fn tokenize(&self) -> impl TokenStream;
}

impl Tokenizable for str {
    fn tokenize(&self) -> impl TokenStream {
        TOKEN_EXPRESSION
            .captures_iter(self)
            .map(|x| x.extract::<0>().0)
            .map(to_token)
            .filter(|x| !matches!(x, Token::Comment))
    }
}
