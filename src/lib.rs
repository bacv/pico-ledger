use async_trait::async_trait;

mod app;
mod repo;

#[derive(Clone, Copy, Debug)]
pub struct Account {
    id: u16,
    available: f32,
    held: f32,
    locked: bool,
}

impl Account {
    pub fn new(client_id: u16) -> Self {
        Self {
            id: client_id,
            available: 0.0,
            held: 0.0,
            locked: false,
        }
    }
    pub fn is_locked(&self) -> bool {
        return self.locked
    }
    pub fn get_client_id(&self) -> u16 {
        return self.id
    }
    pub fn get_available(&self) -> f32 {
        return self.available
    }
    pub fn get_total(&self) -> f32 {
        return self.available + self.held
    }
    pub fn hold(&mut self, amount: f32) {
        self.available -= amount;
        self.held += amount;
    }
    pub fn release(&mut self, amount: f32) {
        self.held -= amount;
        self.available += amount;
    }
    pub fn deposit(&mut self, amount: f32) {
        self.available += amount;
    }
    pub fn withdraw(&mut self, amount: f32) {
        self.available -= amount;
    }
    pub fn withdraw_and_lock(&mut self, amount: f32) {
        self.held -= amount;
        self.locked = true;
    }
}

#[derive(Debug, PartialEq)]
pub struct AccountSummary {
    pub client: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

impl From<&Account> for AccountSummary {
    fn from(a: &Account) -> Self {
        AccountSummary{
            client: a.id,
            available: a.available,
            held: a.held,
            total: a.available + a.held,
            locked: a.locked,
        } 
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BookingState {
    Pristine,
    Normal,
    Disputed,
    Resolved,
    Chargeback,
}

// Booking represents the state of a transaction.
// A transaction that has been charged back or resolved gets locked.
#[derive(Clone, Copy)]
pub struct Booking {
    _tx_id: u32,
    client_id: u16,
    amount: f32,
    locked: bool,
    state: BookingState,
}

impl Booking {
    pub fn new(tx_id: u32, client_id: u16, amount: f32) -> Self {
        Self {
            _tx_id: tx_id,
            client_id,
            amount,
            locked: false,
            state: BookingState::Pristine,
        }
    }
    pub fn set_state(&mut self, state: BookingState) -> &mut Self {
        self.state = state;
        self
    }
    pub fn set_state_and_lock(&mut self, state: BookingState) -> &mut Self {
        self.set_state(state);
        self.locked = true;
        self
    }
    pub fn get_client_id(&self) -> u16 {
        self.client_id
    }
    pub fn get_amount(&self) -> f32 {
        self.amount
    }
    pub fn get_state(&self) -> BookingState {
        self.state
    }
    pub fn is_locked(&self) -> bool {
        self.locked
    }
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
    pub amount: Option<f32>,
}

#[async_trait]
pub trait AccountService {
    async fn dump_accounts(&self) -> LedgerResult<Vec<AccountSummary>>;
}

#[async_trait]
pub trait BookingService {
    async fn process_tx(&self, tx: Tx) -> LedgerResult<()>;
}

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
