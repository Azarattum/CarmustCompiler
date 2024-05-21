mod structure;
mod syntax;

use self::structure::declaration;
use crate::error::*;
use crate::DeclarationStream;
use crate::TokenStream;
use std::iter;

pub trait Analyzable<'a> {
    fn analyze(self) -> impl DeclarationStream<'a>;
}

impl<'a, T: TokenStream<'a> + 'a> Analyzable<'a> for T {
    fn analyze(self) -> impl DeclarationStream<'a> {
        let mut stream = self.peekable();
        iter::from_fn(move || match declaration(&mut stream) {
            Ok(decl) => Some(Ok(decl)),
            Err(error) if error.found.is_some() => Some(Err(error)),
            _ => None,
        })
    }
}
