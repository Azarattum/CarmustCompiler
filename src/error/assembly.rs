use super::ErrorLike;

#[derive(Clone)]
pub struct AssemblyError {
    pub message: String,
}

impl<'a> ErrorLike for AssemblyError {
    fn kind() -> &'static str {
        return "AssemblyError";
    }

    fn slice(&self) -> Option<&str> {
        None
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}
