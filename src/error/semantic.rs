use super::ErrorLike;

#[derive(Clone)]
pub struct SemanticError<'a> {
    pub message: String,
    pub token: Option<&'a str>,
}

impl<'a> ErrorLike for SemanticError<'a> {
    fn kind() -> &'static str {
        return "SemanticError";
    }

    fn slice(&self) -> Option<&'a str> {
        self.token
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}
