use nom::error::{ErrorKind, FromExternalError, ParseError};
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseErrorInfo {
    pub message: String,
    pub input: String,
    pub offset: usize,
}

impl fmt::Display for ParseErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at offset {}: {}", self.offset, self.message)
    }
}

impl<'a> ParseError<&'a str> for ParseErrorInfo {
    fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
        let offset = input.len();
        ParseErrorInfo {
            message: format!("nom error: {:?}", kind),
            input: input.to_string(),
            offset,
        }
    }

    fn append(input: &'a str, kind: ErrorKind, other: Self) -> Self {
        let offset = input.len();
        ParseErrorInfo {
            message: format!("{} -> {:?}", other.message, kind),
            input: input.to_string(),
            offset,
        }
    }

    fn from_char(input: &'a str, c: char) -> Self {
        let offset = input.len();
        ParseErrorInfo {
            message: format!("expected '{}'", c),
            input: input.to_string(),
            offset,
        }
    }

    fn or(self, other: Self) -> Self {
        if self.offset >= other.offset {
            self
        } else {
            other
        }
    }
}

impl<'a, E: fmt::Display> FromExternalError<&'a str, E> for ParseErrorInfo {
    fn from_external_error(input: &'a str, _kind: ErrorKind, e: E) -> Self {
        ParseErrorInfo {
            message: format!("external error: {}", e),
            input: input.to_string(),
            offset: input.len(),
        }
    }
}
