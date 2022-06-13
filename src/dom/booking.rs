
#[derive(Clone, Copy, PartialEq)]
pub enum BookingState {
    Pristine,
    Normal,
    Disputed,
    Resolved,
    Chargeback,
}

// Booking represents the state of a transaction.
// A transaction that has been charged back or resolved gets locked.
#[derive(Clone, Copy)]
pub struct Booking {
    _tx_id: u32,
    client_id: u16,
    amount: i64,
    locked: bool,
    state: BookingState,
}

impl Booking {
    pub fn new(tx_id: u32, client_id: u16, amount: i64) -> Self {
        Self {
            _tx_id: tx_id,
            client_id,
            amount,
            locked: false,
            state: BookingState::Pristine,
        }
    }
    pub fn set_state(&mut self, state: BookingState) -> &mut Self {
        self.state = state;
        self
    }
    pub fn set_state_and_lock(&mut self, state: BookingState) -> &mut Self {
        self.set_state(state);
        self.locked = true;
        self
    }
    pub fn get_client_id(&self) -> u16 {
        self.client_id
    }
    pub fn get_amount(&self) -> i64 {
        self.amount
    }
    pub fn get_state(&self) -> BookingState {
        self.state
    }
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}
