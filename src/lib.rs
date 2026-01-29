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
pub struct JsonError {
    pub error: nojson::JsonParseError,
    pub span: Option<std::ops::Range<usize>>,
    pub text: Option<String>,
    pub backtrace: std::backtrace::Backtrace,
}

#[derive(Debug)]
pub enum Error {
    Fmt(std::fmt::Error),
    Io(std::io::Error),
    Json(Box<JsonError>),
}

impl Error {
    pub fn set_json_text(self, text: impl Into<String>) -> Self {
        match self {
            Error::Json(mut json_err) => {
                if json_err.text.is_none() {
                    json_err.text = Some(text.into());
                }
                Error::Json(json_err)
            }
            other => other,
        }
    }

    pub fn set_json_span(self, value: nojson::RawJsonValue<'_, '_>) -> Self {
        match self {
            Error::Json(mut json_err) => {
                if json_err.span.is_none() {
                    json_err.span =
                        Some(value.position()..value.position() + value.as_raw_str().len());
                }
                Error::Json(json_err)
            }
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
        Error::Json(Box::new(JsonError {
            error,
            span: None,
            text: None,
            backtrace: std::backtrace::Backtrace::capture(),
        }))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Fmt(err) => write!(f, "Formatting error: {}", err),
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Json(json_err) => {
                let JsonError {
                    error,
                    span,
                    text,
                    backtrace,
                } = &**json_err;

                if let Some(text) = text {
                    write!(f, "{}", crate::json::format_parse_error(text, error))?;

                    // Show the range of text with span highlighting
                    if let Some(span) = span
                        && span.start < text.len()
                        && span.end <= text.len()
                    {
                        let error_text = &text[span.clone()];
                        write!(
                            f,
                            "\n  at position {}..{}: \"{}\"",
                            span.start, span.end, error_text
                        )?;
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
            Error::Json(json_err) => Some(&json_err.error),
        }
    }
}
