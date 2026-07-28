use serde::Serialize;

/// Error kind exposed to the frontend so the UI can react specifically
/// (e.g. offer a file picker when Claude Desktop is not found).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    NotFound,
    Validation,
    Io,
    Launch,
    Internal,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    pub detail: Option<String>,
}

impl AppError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            detail: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Validation, message)
    }

    pub fn launch(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Launch, message)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorKind::Io, e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::new(ErrorKind::Internal, format!("config parse error: {e}"))
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AppError", 3)?;
        s.serialize_field("kind", &self.kind)?;
        s.serialize_field("message", &self.message)?;
        s.serialize_field("detail", &self.detail)?;
        s.end()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
