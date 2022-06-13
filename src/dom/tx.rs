#[derive(Clone, Copy)]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Clone, Copy)]
pub struct Tx {
    pub tx_id: u32,
    pub client_id: u16,
    pub tx_type: TxType,
    pub amount: Option<i64>,
}
