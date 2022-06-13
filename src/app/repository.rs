use crate::{Tx, LedgerResult, Account};
use async_trait::async_trait;

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn create_account(&self, client_id: u16) -> LedgerResult<Account>;
    async fn exists(&self, client_id: u16) -> LedgerResult<bool>;
    async fn hold(&self, client_id: u16, amount: f32) -> LedgerResult<()>;
    async fn release(&self, client_id: u16, amount: f32) -> LedgerResult<()>;
    async fn deposit(&self, client_id: u16, amount: f32) -> LedgerResult<()>;
    async fn withdraw(&self, client_id: u16, amount: f32) -> LedgerResult<()>;
    async fn lock(&self, client_id: u16) -> LedgerResult<()>;
    async fn is_locked(&self, client_id: u16) -> LedgerResult<bool>;
    async fn dump_accounts(&self) -> LedgerResult<Vec<Account>>;
}

#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn process_tx(&self, tx: Tx) -> LedgerResult<()>;
}
