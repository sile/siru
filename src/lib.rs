pub mod command_build_doc;
pub mod command_main;
pub mod doc;
pub mod format_item;
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
        span: Option<std::ops::Range<usize>>,
        text: Option<String>,
        backtrace: std::backtrace::Backtrace,
    },
}

impl Error {
    pub fn set_json_text(self, text: impl Into<String>) -> Self {
        match self {
            Error::Json {
                error,
                span,
                text: None,
                backtrace,
            } => Error::Json {
                error,
                span,
                text: Some(text.into()),
                backtrace,
            },
            other => other,
        }
    }

    pub fn set_json_span(self, value: nojson::RawJsonValue<'_, '_>) -> Self {
        match self {
            Error::Json {
                error,
                span: None,
                text,
                backtrace,
            } => Error::Json {
                error,
                span: Some(value.position()..value.position() + value.as_raw_str().len()),
                text,
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
            span: None,
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
                span,
                text,
                backtrace,
            } => {
                if let Some(text) = text {
                    write!(f, "{}", crate::json::format_parse_error(text, error))?;

                    // Show the range of text with span highlighting
                    if let Some(span) = span {
                        if span.start < text.len() && span.end <= text.len() {
                            let error_text = &text[span.clone()];
                            write!(
                                f,
                                "\n  at position {}..{}: \"{}\"",
                                span.start, span.end, error_text
                            )?;
                        }
                    }
                } else {
                    write!(f, "JSON parse error: {}", error)?;
                }

                if backtrace.status() == std::backtrace::BacktraceStatus::Captured {
                    write!(f, "\n\nBACKTRACE:\n{}", backtrace)?;
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
