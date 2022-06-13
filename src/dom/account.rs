use crate::dom::Amount;
use serde::{Serialize, Deserialize};

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountSummary {
    pub client: u16,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

impl From<&Account> for AccountSummary {
    fn from(a: &Account) -> Self {
        AccountSummary{
            client: a.id,
            available: a.available.into(),
            held: a.held.into(),
            total: (a.available + a.held).into(),
            locked: a.locked,
        } 
    }
}


#[cfg(test)]
mod tests {
    use crate::dom::AccountSummary;
    use crate::dom::{Tx, TxType};

    #[test]
    #[ignore]
    fn serialize_tx_csv() {
        let cases: Vec<(&str, AccountSummary)> = vec![
            ("client,available,held,total,locked
1,  1.1,   1.0,    2.1, false
", AccountSummary{client: 1, available: 1_1000.into(), held: 1_0000.into(), total: 2_1000.into(), locked: false}),
        ];

        for (expected, case) in cases.iter() {
            let buf = Vec::new();
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(true)
                .double_quote(true)
                .flexible(true)
                .from_writer(buf);
            wtr.serialize(case).unwrap();
            wtr.flush().unwrap();

            let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
            assert_eq!(expected.to_string(), data);
        }
    }
}