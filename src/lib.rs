use async_trait::async_trait;

mod app;
mod repo;

pub struct Account {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

#[derive(PartialEq)]
pub enum BookingState {
    Pristine,
    Normal,
    Disputed,
    Resolved,
    Chargeback,
}

// Booking represents the state of a transaction.
// A transaction that has been charged back or resolved gets locked.
pub struct Booking {
    pub tx_id: u32,
    pub locked: bool,
    pub state: BookingState,
}

#[derive(Clone, Copy)]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Clone, Copy)]
pub struct Tx {
    pub tx_id: u32,
    pub client_id: u16,
    pub tx_type: TxType,
    pub amount: f32,
}

#[async_trait]
pub trait AccountService {
    async fn dump_accounts(&self) -> LedgerResult<Vec<Account>>;
}

#[async_trait]
pub trait BookingService {
    async fn process_tx(&self, tx: Tx) -> LedgerResult<()>;
}

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
