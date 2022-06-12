use crate::{Tx, LedgerResult, AccountSummary};
use async_trait::async_trait;

#[async_trait]
pub trait AccountRepository {
    async fn create_account(&self, client_id: u16) -> LedgerResult<()>;
    async fn get_account_summary(&self, client_id: u16) -> LedgerResult<AccountSummary>;
}

#[async_trait]
pub trait BookingRepository {
    async fn process_tx(&self, tx: Tx) -> LedgerResult<()>;
}
