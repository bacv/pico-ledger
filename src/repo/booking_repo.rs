use std::{sync::Arc, collections::HashMap};

use async_trait::async_trait;
use futures::lock::Mutex;

use crate::{app::{AccountRepository}, TxType, Tx, LedgerError, BookingState};
use crate::Booking;
use crate::LedgerResult;
use crate::app::BookingRepository;

pub struct LedgerBookingRepository {
    account_repo: Arc<Mutex<dyn AccountRepository>>,
    bookings: Mutex<HashMap<u32, Booking>>,
}

#[async_trait]
impl BookingRepository for LedgerBookingRepository {
    async fn process_tx(&mut self, tx: Tx) -> LedgerResult<()> {
        // Check if account exists, if not create a new one.
        let account = self.account_repo.lock().await
            .get_or_create_account(tx.client_id).await?;

        // Check if account is locked.
        if account.is_locked() {
            return booking_err("account is locked");
        }

        // Check if booking exists and is unlocked. Return an error if locked.
        let mut booking = self.get_or_create_booking(tx).await?;
        if booking.is_locked() {
            return booking_err("booking is locked");
        }
            

        // Check previous booking state just in case we are dealing with 
        // two transactions with the same action.
        match tx.tx_type {
            // Deposit if booking is pristine.
            TxType::Deposit => {
                is_allowed_state(booking, BookingState::Pristine)?;
                self.account_repo.lock().await
                    .deposit(booking.get_client_id(), booking.get_amount()).await?;
                booking.set_state(BookingState::Normal);
            },

            // Account repo decides if withdrawal is possible.
            // *Assuming* that withdrawal can't be disputed.
            TxType::Withdrawal => {
                is_allowed_state(booking, BookingState::Pristine)?;
                self.account_repo.lock().await
                    .withdraw(booking.get_client_id(), booking.get_amount()).await?;
                booking.set_state_and_lock(BookingState::Normal);
            },

            // Dispute is handled by `hold` in account repo.
            TxType::Dispute => {
                is_allowed_state(booking, BookingState::Normal)?;
                self.account_repo.lock().await
                    .hold(booking.get_client_id(), booking.get_amount()).await?;
                booking.set_state(BookingState::Disputed);
            },

            // Resolve is handled by `release` in account repo.
            TxType::Resolve => {
                is_allowed_state(booking, BookingState::Disputed)?;
                self.account_repo.lock().await
                    .release(booking.get_client_id(), booking.get_amount()).await?;
                booking.set_state_and_lock(BookingState::Resolved);
            },

            // Chargeback is handled by `withdraw` in account repo.
            // Account needs to be locked if this happens.
            TxType::Chargeback => {
                is_allowed_state(booking, BookingState::Disputed)?;
                self.account_repo.lock().await
                    .withdraw_and_lock(booking.get_client_id(), booking.get_amount()).await?;
                booking.set_state_and_lock(BookingState::Chargeback);
            },
        };

        self.update_booking(tx.tx_id, booking).await
    }
}

impl LedgerBookingRepository {
    async fn get_or_create_booking(&self, tx: Tx) -> LedgerResult<Booking> {
        let mut repo = self.bookings.lock().await;
        let b = match repo.get(&tx.tx_id) {
            Some(b) => b.clone(),
            None => {
                let b = Booking::new(tx.tx_id, tx.client_id, tx.amount);
                repo.insert(tx.tx_id, b);
                b
            },
        };

        Ok(b)
    }
    async fn update_booking(&mut self, tx_id: u32, booking: Booking) -> LedgerResult<()> {
        self.bookings.lock().await.insert(tx_id, booking);

        Ok(())
    }
}

fn is_allowed_state(current_booking: Booking, expected_state: BookingState) -> LedgerResult<()> {
    if current_booking.get_state() != expected_state {
        return booking_err("transaction is not allowed");
    }

    Ok(())
}

fn booking_err(msg: &str) -> Result<(), LedgerError> {
    Err(LedgerError::repository_error(msg))
}
