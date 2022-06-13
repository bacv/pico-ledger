use super::{LedgerResult, AccountSummary, Tx};
use async_trait::async_trait;

#[async_trait]
pub trait AccountService {
    async fn dump_accounts(&self) -> LedgerResult<Vec<AccountSummary>>;
}

#[async_trait]
pub trait BookingService {
    async fn process_tx(&self, tx: Tx) -> LedgerResult<()>;
}
