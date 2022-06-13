use crate::AccountSummary;
use crate::{Tx, LedgerResult, Account};
use async_trait::async_trait;

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn get_or_create_account(&mut self, client_id: u16) -> LedgerResult<Account>;
    async fn hold(&mut self, client_id: u16, amount: i64) -> LedgerResult<()>;
    async fn release(&mut self, client_id: u16, amount: i64) -> LedgerResult<()>;
    async fn deposit(&mut self, client_id: u16, amount: i64) -> LedgerResult<()>;
    async fn withdraw(&mut self, client_id: u16, amount: i64) -> LedgerResult<()>;
    async fn withdraw_and_lock(&mut self, client_id: u16, amount: i64) -> LedgerResult<()>;
    async fn dump_accounts(&self) -> LedgerResult<Vec<AccountSummary>>;
}

#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn process_tx(&mut self, tx: Tx) -> LedgerResult<()>;
}
