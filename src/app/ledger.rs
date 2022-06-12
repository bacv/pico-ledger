use crate::{LedgerResult, Tx, BookingService, AccountService, AccountSummary};
use std::sync::Arc;

use async_trait::async_trait;

use super::repository::{BookingRepository, AccountRepository};

pub struct Ledger {
    account_repo: Arc<dyn AccountRepository + Send + Sync>,
    booking_repo: Arc<dyn BookingRepository + Send + Sync>
}

#[async_trait]
impl AccountService for Ledger {
    async fn get_account_summary(&self, client_id: u16) -> LedgerResult<AccountSummary> {
        self.account_repo.get_account_summary(client_id).await
    }
}

#[async_trait]
impl BookingService for Ledger {
    async fn process_tx (&self, tx: Tx) -> LedgerResult<()> {
        self.booking_repo.process_tx(tx).await
    }
}