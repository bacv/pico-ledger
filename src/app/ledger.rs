use crate::dom::{AccountSummary, LedgerResult, Tx, BookingService, AccountService};
use std::sync::Arc;

use async_trait::async_trait;
use futures::lock::Mutex;

use super::repository::{BookingRepository, AccountRepository};

pub struct Ledger {
    account_repo: Arc<Mutex<dyn AccountRepository>>,
    booking_repo: Arc<Mutex<dyn BookingRepository>>,
}

impl Ledger {
    pub fn new(
        account_repo: Arc<Mutex<dyn AccountRepository>>,
        booking_repo: Arc<Mutex<dyn BookingRepository>>,
    ) -> Self {
        Self { account_repo, booking_repo }
    }
}

#[async_trait]
impl AccountService for Ledger {
    async fn dump_accounts(&self) -> LedgerResult<Vec<AccountSummary>> {
        self.account_repo.lock().await.dump_accounts().await
    }
}

#[async_trait]
impl BookingService for Ledger {
    async fn process_tx (&self, tx: Tx) -> LedgerResult<()> {
        let mut booking_repo = self.booking_repo.lock().await;
        booking_repo.process_tx(tx).await
    }
}