use super::ErrorLike;

#[derive(Clone)]
pub struct CompileError {
    pub message: &'static str,
}

impl<'a> ErrorLike for CompileError {
    fn kind() -> &'static str {
        return "CompileError";
    }

    fn slice(&self) -> Option<&str> {
        None
    }

    fn message(&self) -> String {
        self.message.to_owned()
    }
}
