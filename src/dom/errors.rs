use std::{fmt::{self, Formatter, Display}, error::Error};

#[derive(Debug)]
pub enum LedgerErrorKind {
    DoesNotExist(String),
    RepositoryError(String),
    ServiceError(String),
}

impl Display for LedgerErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerErrorKind::DoesNotExist(msg) => {
                write!(fmt, "{} does not exist", msg)
            }
            LedgerErrorKind::RepositoryError(msg) => write!(fmt, "Repository error: {}", msg),
            LedgerErrorKind::ServiceError(msg) => write!(fmt, "Service error: {}", msg),
        }
    }
}

impl LedgerErrorKind {
    pub fn into_err(self) -> LedgerError {
        LedgerError {
            kind: self
        }
    }
}

#[derive(Debug)]
pub struct LedgerError {
    pub kind: LedgerErrorKind,
}

impl LedgerError {
    pub fn doesnt_exist<M: Into<String>>(msg: M) -> Self {
        LedgerErrorKind::DoesNotExist(msg.into()).into_err()
    }
    pub fn repository_error<M: Into<String>>(msg: M) -> Self {
        LedgerErrorKind::RepositoryError(msg.into()).into_err()
    }
    pub fn service_error<M: Into<String>>(msg: M) -> Self {
        LedgerErrorKind::ServiceError(msg.into()).into_err()
    }
    pub fn kind(&self) -> &LedgerErrorKind {
        &self.kind
    }
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Error for LedgerError {}

pub type LedgerResult<T> = Result<T, LedgerError>;
