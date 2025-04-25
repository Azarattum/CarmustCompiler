use super::ErrorLike;
use crate::Token;

#[derive(Clone)]
pub struct SyntaxError<'a> {
    pub expected: String,
    pub found: Option<Token<'a>>,
}

impl<'a> ErrorLike for SyntaxError<'a> {
    fn kind() -> &'static str {
        return "SyntaxError";
    }

    fn lookup(text: &str, slice: Option<&str>) -> (usize, usize) {
        let offset = match slice {
            Some(slice) => slice.as_ptr() as usize - text.as_ptr() as usize,
            None => text.len(),
        };

        let line = text[..(offset + 1).min(text.len())].lines().count();
        let start = text[..offset]
            .char_indices()
            .rev()
            .find(|x| x.1 == '\n')
            .and_then(|x| Some(x.0 as i64))
            .unwrap_or(-1);

        return (line - 1, (offset as i64 - start - 1) as usize);
    }

    fn slice(&self) -> Option<&'a str> {
        match self.found {
            Some(
                Token::Identifier(x) | Token::Keyword(x) | Token::Symbol(x) | Token::Data(_, x),
            ) => Some(x),
            _ => None,
        }
    }

    fn message(&self) -> String {
        match self.slice() {
            Some(x) => format!("Expected {}, but found {}!", self.expected, x),
            _ => format!("Unexpected end of file! (expected {})", self.expected),
        }
    }
}
