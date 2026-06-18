use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("operation failed")]
    SilentExit { code: i32 },
}

impl AppError {
    pub fn silent_exit_code(&self) -> Option<i32> {
        match self {
            Self::SilentExit { code } => Some(*code),
            Self::Message(_) => None,
        }
    }
}

pub fn msg(message: impl Into<String>) -> anyhow::Error {
    AppError::Message(message.into()).into()
}

pub fn silent_exit(code: i32) -> anyhow::Error {
    AppError::SilentExit { code }.into()
}
