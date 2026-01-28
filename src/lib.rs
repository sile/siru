pub mod command_build_doc;
pub mod command_main;
pub mod doc;
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
        backtrace: std::backtrace::Backtrace,
    },
}

// From implementations
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
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Fmt(err) => write!(f, "Formatting error: {}", err),
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Json { error, .. } => write!(f, "JSON parse error: {}", error),
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
