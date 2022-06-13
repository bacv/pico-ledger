#[derive(Debug)]
pub enum LedgerErrorKind {
    DoesNotExist(String),
    RepositoryError(String),
    ServiceError(String),
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

pub type LedgerResult<T> = Result<T, LedgerError>;
