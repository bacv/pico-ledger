use crate::dom::LedgerResult;
use std::{sync::Arc, collections::HashMap};

use async_trait::async_trait;
use futures::lock::Mutex;

use crate::{app::{AccountRepository}, dom::{TxType, Tx, LedgerError, BookingState}};
use crate::dom::Booking;
use crate::app::BookingRepository;

pub struct InMemoryBookingRepository {
    account_repo: Arc<Mutex<dyn AccountRepository>>,
    bookings: Mutex<HashMap<u32, Booking>>,
}

impl InMemoryBookingRepository {
    pub fn new(account_repo: Arc<Mutex<dyn AccountRepository>>) -> Self {
        InMemoryBookingRepository{
            account_repo,
            bookings: Mutex::new(HashMap::default()),
        }
    } 
    async fn get_or_create_booking(&self, tx: Tx) -> LedgerResult<Booking> {
        let mut store = self.bookings.lock().await;
        match store.get(&tx.tx_id) {
            Some(b) => Ok(*b),
            None => {
                // *Assuming* that negative amount is not allowed.
                let amount = tx.amount.ok_or_else(|| booking_err("missing amount"))?.to_i64();
                if amount < 0 {
                    return Err(booking_err("negative amount"));
                }
                let b = Booking::new(tx.tx_id, tx.client_id, amount);
                store.insert(tx.tx_id, b);
                Ok(b)
            },
        }
    }
    async fn update_booking(&mut self, tx_id: u32, booking: Booking) -> LedgerResult<()> {
        self.bookings.lock().await.insert(tx_id, booking);

        Ok(())
    }
}

#[async_trait]
impl BookingRepository for InMemoryBookingRepository {
    async fn process_tx(&mut self, tx: Tx) -> LedgerResult<()> {
        // Check if account exists, if not create a new one.
        let account = self.account_repo.lock().await
            .get_or_create_account(tx.client_id).await?;

        // Check if account is locked.
        if account.is_locked() {
            return wrapped_booking_err("account is locked");
        }

        // Check if booking exists and is unlocked. Return an error if locked.
        let mut booking = self.get_or_create_booking(tx).await?;
        if booking.is_locked() {
            return wrapped_booking_err("booking is locked");
        }

        // Check if booking client_id matches account client_id.
        if booking.get_client_id() != account.get_client_id() {
            return wrapped_booking_err("invalid transaction");
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
            // *Assuming* that chargeback can make the account negative.
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

fn is_allowed_state(current_booking: Booking, expected_state: BookingState) -> LedgerResult<()> {
    if current_booking.get_state() != expected_state {
        return wrapped_booking_err("transaction is not allowed");
    }

    Ok(())
}

fn booking_err(msg: &str) -> LedgerError {
    LedgerError::repository_error(msg)
}

fn wrapped_booking_err(msg: &str) -> Result<(), LedgerError> {
    Err(booking_err(msg))
}

#[cfg(test)]
mod tests {
    use crate::dom::Amount;
use crate::repo::account_repo::InMemoryAccountRepository;
    use crate::dom::AccountSummary;
    use std::cmp::Ordering;
    use super::*;

    struct TestCase {
        txs: Vec<(Tx, bool)>, // true if no errors expected.
        expected: Vec<AccountSummary>,
    }

    struct CommonCases;

    impl CommonCases {
        pub fn new() -> HashMap<&'static str, TestCase> {
                return HashMap::from([
                    ("booking_gets_locked", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Dispute, amount: None}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Resolve, amount: None}, true),
                        ],
                        expected: vec![
                            AccountSummary{client: 1, available: 10_0000.into(), total: 10_0000.into(), held: 0_0000.into(), locked: false}
                        ]
                    }),
                    ("invalid_tx_1", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Dispute, amount: None}, true),
                            (Tx{tx_id: 0, client_id: 1, tx_type: TxType::Resolve, amount: None}, false),
                        ],
                        expected: vec![
                            AccountSummary{client: 1, available: 0_0000.into(), total: 10_0000.into(), held: 10_0000.into(), locked: false}
                        ]
                    }),
                    ("invalid_tx_2", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Dispute, amount: None}, true),
                            (Tx{tx_id: 1, client_id: 2, tx_type: TxType::Resolve, amount: None}, false),
                        ],
                        expected: vec![
                            AccountSummary{client: 2, available: 0_0000.into(), total: 0_0000.into(), held: 0_0000.into(), locked: false},
                            AccountSummary{client: 1, available: 0_0000.into(), total: 10_0000.into(), held: 10_0000.into(), locked: false}
                        ]
                    }),
                    ("deposit_booking", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 2, client_id: 2, tx_type: TxType::Deposit, amount: Some(Amount::from(11_0000))}, true),
                            (Tx{tx_id: 3, client_id: 3, tx_type: TxType::Deposit, amount: Some(Amount::from(12_0000))}, true),
                        ],
                        expected: vec![
                            AccountSummary{client: 1, available: 10_0000.into(), total: 10_0000.into(), held: 0_0000.into(), locked: false},
                            AccountSummary{client: 2, available: 11_0000.into(), total: 11_0000.into(), held: 0_0000.into(), locked: false},
                            AccountSummary{client: 3, available: 12_0000.into(), total: 12_0000.into(), held: 0_0000.into(), locked: false},
                        ]
                    }),
                    ("tx_to_locked_account", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Dispute, amount: None}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Chargeback, amount: None}, true),
                            (Tx{tx_id: 2, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(1_0000))}, false),
                        ],
                        expected: vec![
                            AccountSummary{client: 1, available: 0_0000.into(), total: 0_0000.into(), held: 0_0000.into(), locked: true}
                        ]
                    }),
                    ("multiple_txs_w_same_id", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Dispute, amount: None}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Dispute, amount: None}, false),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Resolve, amount: None}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Resolve, amount: None}, false),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Chargeback, amount: None}, false),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Chargeback, amount: None}, false),
                        ],
                        expected: vec![
                            AccountSummary{client: 1, available: 10_0000.into(), total: 10_0000.into(), held: 0_0000.into(), locked: false}
                        ]
                    }),
                    ("withdraw_booking", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 2, client_id: 1, tx_type: TxType::Withdrawal, amount: Some(Amount::from(5_0000))}, true),
                        ],
                        expected: vec![
                            AccountSummary{client: 1, available: 5_0000.into(), total: 5_0000.into(), held: 0_0000.into(), locked: false}
                        ]
                    }),
                    ("dispute_booking", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 2, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(5_0000))}, true),
                            (Tx{tx_id: 2, client_id: 1, tx_type: TxType::Dispute, amount: None}, true),
                            (Tx{tx_id: 3, client_id: 1, tx_type: TxType::Withdrawal, amount: Some(Amount::from(9_0000))}, true),
                            (Tx{tx_id: 4, client_id: 1, tx_type: TxType::Withdrawal, amount: Some(Amount::from(9_0000))}, false),
                        ],
                        expected: vec![
                            AccountSummary{client: 1, available: 1_0000.into(), total: 6_0000.into(), held: 5_0000.into(), locked: false}
                        ]
                    }),
                    ("resolve_booking", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Dispute, amount: None}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Resolve, amount: None}, true),
                        ],
                        expected: vec![
                            AccountSummary{client: 1, available: 10_0000.into(), total: 10_0000.into(), held: 0_0000.into(), locked: false}
                        ]
                    }),
                    ("chargeback_booking", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 2, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(11_0000))}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Dispute, amount: None}, true),
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Chargeback, amount: None}, true),
                        ],
                        expected: vec![
                            AccountSummary{client: 1, available: 11_0000.into(), total: 11_0000.into(), held: 0_0000.into(), locked: true}
                        ]
                    }),
                    ("negative_chargeback_booking", TestCase {
                        txs: vec![
                            (Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(10_0000))}, true),
                            (Tx{tx_id: 2, client_id: 1, tx_type: TxType::Deposit, amount: Some(Amount::from(11_0000))}, true),
                            (Tx{tx_id: 3, client_id: 1, tx_type: TxType::Withdrawal, amount: Some(Amount::from(20_0000))}, true),
                            (Tx{tx_id: 2, client_id: 1, tx_type: TxType::Dispute, amount: None}, true), // available -10; held 11; total 1
                            (Tx{tx_id: 4, client_id: 1, tx_type: TxType::Withdrawal, amount: Some(Amount::from(1_0000))}, false), // fails because of negative available balance
                            (Tx{tx_id: 2, client_id: 1, tx_type: TxType::Chargeback, amount: None}, true), // available -10; held 0; total -10
                        ],
                        expected: vec![
                            AccountSummary{client: 1, available: (-10_0000).into(), total: (-10_0000).into(), held: 0_0000.into(), locked: true}
                        ]
                    }),
                ])
        }
    }

    fn new_booking_account_repo_pair() -> (InMemoryBookingRepository, Arc<Mutex<InMemoryAccountRepository>>) {
        let account_repo = Arc::new(Mutex::new(InMemoryAccountRepository::new()));
        (InMemoryBookingRepository::new(account_repo.clone()), account_repo)
    }

    #[tokio::test]
    async fn booking_when_tx_without_existing_account() {
        let tx = Tx{
            tx_id: 1,
            client_id: 1,
            tx_type: TxType::Deposit,
            amount: Some(Amount::from(10_0000)),
        };
        
        let (mut booking_repo, account_repo) = new_booking_account_repo_pair();

        let res = booking_repo.process_tx(tx).await;
        assert!(res.is_ok());

        let accounts = account_repo.lock().await.dump_accounts().await.unwrap();
        assert_eq!(tx.client_id, accounts[0].client);
        assert_eq!(tx.amount, Some(accounts[0].available.into()));
        assert_eq!(tx.amount, Some(accounts[0].total.into()));
        assert_eq!(Amount::from(0_0000), accounts[0].held);
        assert_eq!(false, accounts[0].locked);
    }

    #[tokio::test]
    async fn common_cases()  {
        let cases = CommonCases::new();
        for (title, mut case) in cases.into_iter() {
            let (mut booking_repo, account_repo) = new_booking_account_repo_pair();

            for (tx, should_succeed) in case.txs {
                let res = booking_repo.process_tx(tx).await;
                assert!(should_succeed == res.is_ok(), "{}: tx_id: {}", title, tx.tx_id);
            }

            assert_eq!(
                case.expected.sort_by(summary_sort),
                account_repo.lock().await.dump_accounts().await.unwrap().sort_by(summary_sort),
                 "{}", title
            );
        }
    }

    fn summary_sort(a: &AccountSummary, b: &AccountSummary) -> Ordering {
        a.client.partial_cmp(&b.client).unwrap()
    }
}