use std::{sync::Arc, collections::HashMap};

use async_trait::async_trait;

use crate::{app::{AccountRepository, BookingRepository}, TxType, Tx, LedgerResult, Booking, LedgerError, LedgerErrorKind, BookingState};

pub struct LedgerBookingRepository {
    account_repo: Arc<dyn AccountRepository>,
    bookings: HashMap<u32, Booking>,
}

#[async_trait]
impl BookingRepository for LedgerBookingRepository {
    async fn process_tx(&self, tx: Tx) -> LedgerResult<()> {
        // Check if account exists, if not create a new one.
        if !self.account_repo.exists(tx.client_id).await? {
            self.account_repo.create_account(tx.client_id).await?;
        }

        // Check if account is locked.
        if self.account_repo.is_locked(tx.client_id).await? {
            return Err(LedgerError::repository_error("account is locked"));
        }

        // Check if booking exists and is unlocked. Return an error if locked.
        if self.exists(tx.tx_id).await? {
            if self.is_locked(tx.tx_id).await? {
                return Err(LedgerError::repository_error("booking is locked"));
            }
        } else {
            // If transaction has a type of dispute/resolve/chargeback
            // and booking does not exist, return an error.
            match tx.tx_type {
                TxType::Deposit | TxType::Resolve | TxType::Chargeback =>
                    return Err(LedgerError::repository_error("non existent transaction")),
                _ => self.create_booking(tx).await?,
            };
        }

        // Check previous booking state just in case we are dealing with 
        // two transactions with the same action.
        let booking_state = self.get_state(tx.tx_id).await?;

        match tx.tx_type {
            // Deposit if booking is pristine.
            TxType::Deposit => {
                is_allowed_state(booking_state, BookingState::Pristine)?;
                self.account_repo.deposit(tx.client_id, tx.amount).await?;
                self.set_state(tx.tx_id, BookingState::Normal).await
            },

            // Account repo decides if withdrawal is possible.
            // *Assuming* that withdrawal can't be disputed.
            TxType::Withdrawal => {
                is_allowed_state(booking_state, BookingState::Pristine)?;
                self.account_repo.withdraw(tx.client_id, tx.amount).await?;
                self.set_state(tx.tx_id, BookingState::Normal).await?;
                self.lock(tx.tx_id).await
            },

            // Dispute is handled by `hold` in account repo.
            TxType::Dispute => {
                is_allowed_state(booking_state, BookingState::Normal)?;
                self.account_repo.hold(tx.client_id, tx.amount).await?;
                self.set_state(tx.tx_id, BookingState::Disputed).await
            },

            // Resolve is handled by `release` in account repo.
            TxType::Resolve => {
                is_allowed_state(booking_state, BookingState::Disputed)?;
                self.account_repo.release(tx.client_id, tx.amount).await?;
                self.set_state(tx.tx_id, BookingState::Resolved).await?;
                self.lock(tx.tx_id).await
            },

            // Chargeback is handled by `withdraw` in account repo.
            // Account needs to be locked if this happens.
            TxType::Chargeback => {
                is_allowed_state(booking_state, BookingState::Disputed)?;
                self.account_repo.withdraw(tx.client_id, tx.amount).await?;
                self.account_repo.lock(tx.client_id).await?;
                self.set_state(tx.tx_id, BookingState::Chargeback).await?;
                self.lock(tx.tx_id).await
            },
        }
    }
}

impl LedgerBookingRepository {
    async fn create_booking(&self, tx: Tx) -> LedgerResult<()> {
        todo!()
    }
    async fn exists(&self, tx_id: u32) -> LedgerResult<bool> {
        todo!()
    }
    async fn lock(&self, tx_id: u32) -> LedgerResult<()> {
        todo!()
    }
    async fn is_locked(&self, tx_id: u32) -> LedgerResult<bool> {
        todo!()
    }
    async fn get_state(&self, tx_id: u32) -> LedgerResult<BookingState> {
        todo!()
    }
    async fn set_state(&self, tx_id: u32, state: BookingState) -> LedgerResult<()> {
        todo!()
    }
}

fn is_allowed_state(current: BookingState, expected: BookingState) -> LedgerResult<()> {
    if current != expected {
        return Err(LedgerError::repository_error("transaction is not allowed"));
    }

    Ok(())
}