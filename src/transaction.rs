use rust_decimal::Decimal;
use serde::Deserialize;

use crate::decimal::deserialize_option_decimal;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, PartialEq, Default)]
pub enum TransactionStatus {
    #[default]
    Normal,
    Disputed,
    Resolved,
    Chargebacked,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct TransactionEntity {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub client: u16,
    pub tx: u32,
    #[serde(deserialize_with = "deserialize_option_decimal")]
    pub amount: Option<Decimal>,
}

#[derive(Debug, PartialEq, Default)]
pub struct Transaction {
    pub amount: Option<Decimal>,
    pub status: TransactionStatus,
}

impl From<&TransactionEntity> for Transaction {
    fn from(entity: &TransactionEntity) -> Self {
        Transaction {
            amount: entity.amount,
            status: TransactionStatus::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use csv::ReaderBuilder;

    use super::*;

    fn deserialize_from_string(input: &str) -> Vec<TransactionEntity> {
        let mut result = Vec::new();
        let mut rdr = ReaderBuilder::new().from_reader(input.as_bytes());
        for record in rdr.deserialize::<TransactionEntity>() {
            match record {
                Ok(transaction) => result.push(transaction),
                Err(err) => eprintln!("Error deserializing transaction: {}", err),
            }
        }
        
        result
    }

    #[test]
    fn test_deserialize_transactions() {
        let expected = vec![TransactionEntity {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(Decimal::from_str("100.00").unwrap()),
        }, TransactionEntity {
            transaction_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(Decimal::from_str("100.00").unwrap()),
        }, TransactionEntity {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 3,
            amount: None,
        }, TransactionEntity {
            transaction_type: TransactionType::Resolve,
            client: 1,
            tx: 4,
            amount: None,
        }, TransactionEntity {
            transaction_type: TransactionType::Chargeback,
            client: 1,
            tx: 5,
            amount: None,
        }];

        assert_eq!(deserialize_from_string("type,client,tx,amount\ndeposit,1,1,100\nwithdrawal,1,2,100\ndispute,1,3,\nresolve,1,4,\nchargeback,1,5,"), expected);
    }

    #[test]
    fn test_deserialize_transaction_with_invalid_type() {
        let input = "type,client,tx,amount\ntest,1,1,100";
        let expected = vec![];

        assert_eq!(deserialize_from_string(input), expected);
    }
}
