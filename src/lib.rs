use async_trait::async_trait;

mod app;

pub struct Account {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

pub struct AccountSummary{}

pub enum BookingState {
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

pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

pub struct Tx {
    pub tx_id: u32,
    pub account_id: u16,
    pub tx_type: TxType,
    pub amount: f32,
}

#[async_trait]
pub trait AccountService {
    async fn get_account_summary(&self, client_id: u16) -> LedgerResult<AccountSummary>;
}

#[async_trait]
pub trait BookingService {
    async fn process_tx(&self, tx: Tx) -> LedgerResult<()>;
}

pub enum LedgerErrorKind {
    AccountServiceError(String),
    BookingServiceError(String),
}

pub struct LedgerError {
    pub kind: LedgerErrorKind,
}

pub type LedgerResult<T> = Result<T, LedgerError>;
