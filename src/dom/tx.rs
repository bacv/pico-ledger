use crate::dom::Amount;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Tx {
    #[serde(rename = "tx")]
    pub tx_id: u32,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "type")]
    pub tx_type: TxType,
    #[serde(rename = "amount")]
    pub amount: Option<Amount>,
}

#[cfg(test)]
mod tests {
    use crate::dom::{Tx, TxType};

    #[test]
    fn deserialize_tx_csv() {
        let cases: Vec<(&str, Tx)> = vec![
            ("type,client,tx,amount
deposit,1,1,1.0004", Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(1_0004.into())}),
("type, client, tx, amount
dispute,    1,  1", Tx{tx_id: 1, client_id: 1, tx_type: TxType::Dispute, amount: None}),
        ];

        for (case, expected) in cases.iter() {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .trim(csv::Trim::All)
                .from_reader(case.as_bytes());
            for record in rdr.deserialize() {
                let r: Tx = record.map_err(|e| println!("{}", e)).unwrap();
                assert_eq!(expected.amount, r.amount);
                assert_eq!(expected.tx_type, r.tx_type);
                assert_eq!(expected.tx_id, r.tx_id);
                assert_eq!(expected.client_id, r.client_id);
            }
        }
    }
    
    #[test]
    #[ignore]
    fn serialize_tx_csv() {
        let cases: Vec<(&str, Tx)> = vec![
            ("type,client,tx,amount
deposit,1,1,1.0004
", Tx{tx_id: 1, client_id: 1, tx_type: TxType::Deposit, amount: Some(1_0004.into())}),
("type,client,tx,amount
dispute,1,1", Tx{tx_id: 1, client_id: 1, tx_type: TxType::Dispute, amount: None}),
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