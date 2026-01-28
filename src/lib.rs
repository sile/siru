pub mod command_build_doc;
pub mod command_main;
pub mod doc;
pub mod format_type;
pub mod item_view;
pub mod json;
pub mod markdown;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Fmt(std::fmt::Error),
    Io(std::io::Error),
    Json {
        error: nojson::JsonParseError,
        text: Option<String>,
        backtrace: std::backtrace::Backtrace,
    },
}

impl Error {
    pub fn set_json_text(self, text: impl Into<String>) -> Self {
        match self {
            Error::Json {
                error,
                text: None,
                backtrace,
            } => Error::Json {
                error,
                text: Some(text.into()),
                backtrace,
            },
            other => other,
        }
    }
}

impl From<std::fmt::Error> for Error {
    fn from(error: std::fmt::Error) -> Self {
        Error::Fmt(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<nojson::JsonParseError> for Error {
    fn from(error: nojson::JsonParseError) -> Self {
        Error::Json {
            error,
            text: None,
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Fmt(err) => write!(f, "Formatting error: {}", err),
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Json {
                error,
                text,
                backtrace,
            } => {
                if let Some(text) = text {
                    write!(f, "{}", crate::json::format_parse_error(text, error))
                } else {
                    write!(f, "JSON parse error: {}", error)
                }?;
                if backtrace.status() == std::backtrace::BacktraceStatus::Captured {
                    write!(f, "\n{}", backtrace)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Fmt(err) => Some(err),
            Error::Io(err) => Some(err),
            Error::Json { error, .. } => Some(error),
        }
    }
}
