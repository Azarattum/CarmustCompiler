mod expression;
mod structure;
mod syntax;

use crate::ast::Statement;
use crate::error::*;
use crate::TokenStream;
use structure::block;

pub trait Analyzable<'a> {
    fn analyze(self) -> Result<Vec<Statement<'a>>, SyntaxError<'a>>;
}

impl<'a, T: TokenStream<'a> + 'a> Analyzable<'a> for T {
    fn analyze(self) -> Result<Vec<Statement<'a>>, SyntaxError<'a>> {
        let mut stream = self.peekable();
        block(&mut stream, "")
    }
}
