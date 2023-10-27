// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    InvalidFileSignature,
    Bug,
    InvalidArgument,
    Io,
    Memory,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for DatrieError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.msg)
    }
}

impl std::error::Error for DatrieError {}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DatrieError {
    kind: ErrorKind,
    msg: String,
}

impl DatrieError {
    pub fn new(kind: ErrorKind, msg: String) -> Self {
        Self { kind, msg }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn msg(&self) -> &str {
        self.msg.as_str()
    }
}

impl From<std::io::Error> for DatrieError {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorKind::Bug, format!("std::io::Error: {}", e))
    }
}

impl From<std::string::FromUtf8Error> for DatrieError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::new(ErrorKind::Bug, format!("std::string::FromUtf8Error: {}", e))
    }
}
