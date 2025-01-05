use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rust_decimal::Decimal;
use serde::Serialize;
use tokio::sync::mpsc;
use std::error::Error;

use crate::decimal::serialize_decimal;
use crate::transaction::{Transaction, TransactionEntity, TransactionStatus, TransactionType};

#[derive(Debug, Serialize)]
pub struct AccountEntity {
    pub client: u16,
    #[serde(serialize_with = "serialize_decimal")]
    pub available: Decimal,
    #[serde(serialize_with = "serialize_decimal")]
    pub held: Decimal,
    #[serde(serialize_with = "serialize_decimal")]
    pub total: Decimal,
    pub locked: bool,
}

pub struct Account {
    client: u16,
    held: Decimal,
    total: Decimal,
    locked: bool,

    transactions: HashMap<u32, Transaction>,
}

impl From<&Account> for AccountEntity {
    fn from(account: &Account) -> Self {
        AccountEntity {
            client: account.client,
            available: account.available(),
            held: account.held,
            total: account.total,
            locked: account.locked,
        }
    }
}

impl Account {
    pub fn new(client: u16) -> Self {
        Account {
            client,
            held: Decimal::new(0, 0),
            total: Decimal::new(0, 0),
            locked: false,
            transactions: HashMap::new(),
        }
    }

    pub fn available(&self) -> Decimal {
        self.total - self.held
    }

    pub fn add_transaction(&mut self, tx: u32, transaction: Transaction) {
        self.transactions.insert(tx, transaction);
    }

    // Getters
    pub fn client(&self) -> u16 {
        self.client
    }

    pub fn held(&self) -> Decimal {
        self.held
    }

    pub fn total(&self) -> Decimal {
        self.total
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    // Setters
    pub fn set_held(&mut self, held: Decimal) {
        self.held = held;
    }

    pub fn set_total(&mut self, total: Decimal) {
        self.total = total;
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn process_transaction(&mut self, transaction_entity: TransactionEntity) -> Result<(), Box<dyn Error>> {
        match transaction_entity.transaction_type {
            TransactionType::Deposit => self.handle_deposit(&transaction_entity),
            TransactionType::Withdrawal => self.handle_withdrawal(&transaction_entity),
            TransactionType::Dispute => self.handle_dispute(&transaction_entity),
            TransactionType::Resolve => self.handle_resolve(&transaction_entity),
            TransactionType::Chargeback => self.handle_chargeback(&transaction_entity),
        }
    }

    fn handle_deposit(&mut self, transaction_entity: &TransactionEntity) -> Result<(), Box<dyn Error>> {
        if self.locked() {
            return Err("Account is locked".into());
        }
        
        self.total += transaction_entity.amount.unwrap_or(Decimal::new(0, 0));

        // If I correctly understand the task, the only deposit transactions could be disputed
        // so we save only deposit transactions
        self.add_transaction(transaction_entity.tx, Transaction::from(transaction_entity));

        Ok(())
    }

    fn handle_withdrawal(&mut self, transaction_entity: &TransactionEntity) -> Result<(), Box<dyn Error>> {
        let amount = transaction_entity.amount.unwrap_or(Decimal::new(0, 0));

        if self.locked() {
            return Err("Account is locked".into());
        }

        if amount.is_zero() || amount.is_sign_negative() || amount > self.available() {
            return Err("Withdrawal amount is invalid".into());
        }

        self.total -= amount;

        Ok(())
    }

    fn handle_dispute(&mut self, transaction_entity: &TransactionEntity) -> Result<(), Box<dyn Error>> {
        if self.locked() {
            return Err("Account is locked".into());
        }

        let available = self.available();
        let disputed_tx = match self.transactions.get_mut(&transaction_entity.tx) {
            Some(tx) => tx,
            None => return Err("Transaction not found".into()),
        };

        if disputed_tx.status != TransactionStatus::Normal {
            return Err("Transaction is already disputed".into());
        }

        let amount = disputed_tx.amount.unwrap_or(Decimal::new(0, 0));
        if amount > available {
            return Err("Transaction amount is greater than available funds".into());
        }

        disputed_tx.status = TransactionStatus::Disputed;
        self.held += amount;

        Ok(())
    }

    fn handle_resolve(&mut self, transaction_entity: &TransactionEntity) -> Result<(), Box<dyn Error>> {
        if self.locked() {
            return Err("Account is locked".into());
        }

        let disputed_tx = match self.transactions.get_mut(&transaction_entity.tx) {
            Some(tx) => tx,
            None => return Err("Transaction not found".into()),
        };

        if disputed_tx.status != TransactionStatus::Disputed {
            return Err("Transaction is not disputed".into());
        }

        disputed_tx.status = TransactionStatus::Resolved;
        self.held -= disputed_tx.amount.unwrap_or(Decimal::new(0, 0));

        Ok(())
    }

    fn handle_chargeback(&mut self, transaction_entity: &TransactionEntity) -> Result<(), Box<dyn Error>> {
        if self.locked() {
            return Err("Account is locked".into());
        }

        let disputed_tx = match self.transactions.get_mut(&transaction_entity.tx) {
            Some(tx) => tx,
            None => return Err("Transaction not found".into()),
        };

        if disputed_tx.status != TransactionStatus::Disputed {
            return Err("Transaction is not disputed".into());
        }

        disputed_tx.status = TransactionStatus::Chargebacked;
        self.held -= disputed_tx.amount.unwrap_or(Decimal::new(0, 0));
        self.total -= disputed_tx.amount.unwrap_or(Decimal::new(0, 0));
        self.locked = true;

        Ok(())
    }
}

pub enum AccountWorkerMessage {
    Transaction(TransactionEntity),
    Shutdown,
}

pub struct AccountWorker {
    pub account: Arc<RwLock<Account>>,
    receiver: mpsc::Receiver<AccountWorkerMessage>,
}

impl AccountWorker {
    pub fn new(receiver: mpsc::Receiver<AccountWorkerMessage>, account: Arc<RwLock<Account>>) -> Self {
        Self {
            account: account,
            receiver,
        }
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                AccountWorkerMessage::Transaction(tx) => {
                    let mut account = self.account.write().await;

                    if let Err(e) = account.process_transaction(tx) {
                        eprintln!("Error processing transaction: {}", e);
                    }
                }
                AccountWorkerMessage::Shutdown => break,
            }
        }
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use csv::WriterBuilder;

    fn serialize_to_string(account: &AccountEntity) -> String {
        let mut wtr = WriterBuilder::new().from_writer(vec![]);
        wtr.serialize(account).unwrap();
        String::from_utf8(wtr.into_inner().unwrap()).unwrap()
    }

    #[test]
    fn test_account_entity_from() {
        let mut account = Account::new(1);
        account.set_total(dec!(100.0));
        account.set_held(dec!(30.0));
        account.lock();

        let entity = AccountEntity::from(&account);

        assert_eq!(entity.client, 1);
        assert_eq!(entity.total, dec!(100.0));
        assert_eq!(entity.held, dec!(30.0));
        assert_eq!(entity.available, dec!(70.0));
        assert!(entity.locked);
    }

    #[test]
    fn test_serialize_account_entity() {
        let mut account = Account::new(1);
        account.set_total(dec!(100.0));
        account.set_held(dec!(30.0));

        let entity = AccountEntity::from(&account);
        assert_eq!(
            serialize_to_string(&entity),
            "client,available,held,total,locked\n1,70.0,30.0,100.0,false\n"
        );
    }

    #[test]
    fn test_serialize_account_entity_with_zero_values() {
        let account = Account::new(1);
        let entity = AccountEntity::from(&account);
        
        assert_eq!(
            serialize_to_string(&entity),
            "client,available,held,total,locked\n1,0,0,0,false\n"
        );
    }

    #[test]
    fn test_serialize_account_entity_locked() {
        let mut account = Account::new(1);
        account.set_total(dec!(100.0));
        account.lock();

        let entity = AccountEntity::from(&account);
        assert_eq!(
            serialize_to_string(&entity),
            "client,available,held,total,locked\n1,100.0,0,100.0,true\n"
        );
    }

    #[test]
    fn test_serialize_account_entity_with_high_precision() {
        let mut account = Account::new(12);
        account.set_total(dec!(100.123456789123));
        account.set_held(dec!(30.123456799123));


        let entity = AccountEntity::from(&account);
        assert_eq!(
            serialize_to_string(&entity),
            "client,available,held,total,locked\n12,69.9999,30.1234,100.1234,false\n"
        );
    }

    #[test]
    fn test_account_available_calculation() {
        let mut account = Account::new(1);
        account.set_total(dec!(100.0));
        account.set_held(dec!(30.0));

        assert_eq!(account.available(), dec!(70.0));
    }
}