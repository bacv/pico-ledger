use serde::Serializer;
use serde::Deserializer;
use std::fmt;
use serde::{de, Serialize, Deserialize};

const PRECISION: i64 = 10000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount(i64);

impl Amount {
    pub fn to_i64(&self) -> i64 {
        self.0
    }
}

impl From<i64> for Amount {
    fn from(item: i64) -> Self {
        Amount(item)
    }
}

impl Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64((self.0 as f64) / PRECISION as f64)
    }
}

struct AmountVisitor;

impl<'de> de::Visitor<'de> for AmountVisitor {
    type Value = Amount;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // When deserializing f64 it's multiplied by required precision points
        // and only integer part is used without any rounding.
        // TODO: error checks.
        Ok(Amount((value * PRECISION as f64) as i64))
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_f64(AmountVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::dom::Amount;

    #[test]
    fn deserialize_amount_csv() {
        let cases: Vec<(&str, i64)> = vec![
            ("1.2", 12000),
            ("1.2345", 12345),
            ("1.00", 10000),
            ("1", 10000),
            ("12345.12345678", 12345_1234),
        ];

        for (c, e) in cases.iter() {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(c.as_bytes());
            for record in rdr.deserialize() {
                let r: Amount = record.unwrap();
                assert_eq!(*e, r.0);
            }
        }
    }

    #[test]
    fn serialize_amount_csv() {
        let cases: Vec<(&str, Amount)> = vec![
            ("1.2\n", Amount::from(1_2000)),
            ("1.2345\n", Amount::from(1_2345)),
            ("1.0\n", Amount::from(1_0000)),
            ("1234.0\n", Amount::from(1234_0000)),
            ("0.1234\n", Amount::from(1234)),
        ];

        for (e, c) in cases.iter() {
            let buf = Vec::new();
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(buf);
            wtr.serialize(c).unwrap();
            wtr.flush().unwrap();
            let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
            assert_eq!(e.to_string(), data);
        }
    }
}