
#[derive(Clone, Copy, Debug)]
pub struct Account {
    id: u16,
    available: i64,
    held: i64,
    locked: bool,
}

impl Account {
    pub fn new(client_id: u16) -> Self {
        Self {
            id: client_id,
            available: 0,
            held: 0,
            locked: false,
        }
    }
    pub fn is_locked(&self) -> bool {
        return self.locked
    }
    pub fn get_client_id(&self) -> u16 {
        return self.id
    }
    pub fn get_available(&self) -> i64 {
        return self.available
    }
    pub fn get_total(&self) -> i64 {
        return self.available + self.held
    }
    pub fn hold(&mut self, amount: i64) {
        self.available -= amount;
        self.held += amount;
    }
    pub fn release(&mut self, amount: i64) {
        self.held -= amount;
        self.available += amount;
    }
    pub fn deposit(&mut self, amount: i64) {
        self.available += amount;
    }
    pub fn withdraw(&mut self, amount: i64) {
        self.available -= amount;
    }
    pub fn withdraw_and_lock(&mut self, amount: i64) {
        self.held -= amount;
        self.locked = true;
    }
}

#[derive(Debug, PartialEq)]
pub struct AccountSummary {
    pub client: u16,
    pub available: i64,
    pub held: i64,
    pub total: i64,
    pub locked: bool,
}

impl From<&Account> for AccountSummary {
    fn from(a: &Account) -> Self {
        AccountSummary{
            client: a.id,
            available: a.available,
            held: a.held,
            total: a.available + a.held,
            locked: a.locked,
        } 
    }
}
