use crate::{LedgerResult, Tx, BookingService, AccountService, Account};
use std::sync::Arc;

use async_trait::async_trait;

use super::repository::{BookingRepository, AccountRepository};

pub struct Ledger {
    account_repo: Arc<dyn AccountRepository>,
    booking_repo: Arc<dyn BookingRepository>
}

#[async_trait]
impl AccountService for Ledger {
    async fn dump_accounts(&self) -> LedgerResult<Vec<Account>> {
        self.account_repo.dump_accounts().await
    }
}

#[async_trait]
impl BookingService for Ledger {
    async fn process_tx (&self, tx: Tx) -> LedgerResult<()> {
        self.booking_repo.process_tx(tx).await
    }
}