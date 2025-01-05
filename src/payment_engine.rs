use std::{collections::HashMap, sync::Arc};

use tokio::sync::mpsc;
use tokio::sync::RwLock;
use crate::transaction::TransactionEntity;
use crate::account::{Account, AccountEntity, AccountWorker, AccountWorkerMessage};
use std::error::Error;

const WORKER_CHANNEL_SIZE: usize = 100;

pub struct PaymentEngine {
    account_senders: HashMap<u16, mpsc::Sender<AccountWorkerMessage>>,
    accounts: HashMap<u16, Arc<RwLock<Account>>>,
    spawned_workers: HashMap<u16, tokio::task::JoinHandle<()>>,
}

impl PaymentEngine {
    pub fn new() -> Self {
        PaymentEngine {
            account_senders: HashMap::new(),
            accounts: HashMap::new(),
            spawned_workers: HashMap::new(),
        }
    }

    async fn add_account_if_not_exists(&mut self, client_id: u16) -> mpsc::Sender<AccountWorkerMessage> {
        if let Some(sender) = self.account_senders.get(&client_id) {
            return sender.clone();
        }

        let (tx, rx) = mpsc::channel(WORKER_CHANNEL_SIZE);
        let account_arc = Arc::new(RwLock::new(Account::new(client_id)));
        let worker = AccountWorker::new( rx, account_arc.clone());
        
        let handler = tokio::spawn(async move {
            worker.run().await;
        });

        self.spawned_workers.insert(client_id, handler);
        self.account_senders.insert(client_id, tx.clone());
        self.accounts.insert(client_id, account_arc);
        tx
    }

    pub async fn get_account_entities(&self, order: bool) -> Vec<AccountEntity> {
        let mut account_entities = Vec::new();

        for (_, account) in self.accounts.iter() {
            let account_guard = account.read().await;
            let account_entity = AccountEntity::from(&*account_guard);
            account_entities.push(account_entity);
        }

        if order {
            account_entities.sort_by_key(|a| a.client);
        }

        account_entities
    }

    pub async fn process_transaction(&mut self, transaction_entity: TransactionEntity) -> Result<(), Box<dyn Error>> {
        let account_sender = self.add_account_if_not_exists(transaction_entity.client).await;

        account_sender.send(AccountWorkerMessage::Transaction(transaction_entity)).await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) {
        // First send shutdown message to all workers
        for (_, sender) in self.account_senders.iter_mut() {
            if let Err(e) = sender.send(AccountWorkerMessage::Shutdown).await {
                eprintln!("Failed to send shutdown message: {}", e);
            }
        }

        // Then wait for all workers to complete
        for (client_id, handle) in self.spawned_workers.drain() {
            if let Err(e) = handle.await {
                eprintln!("Worker for client {} failed to shutdown: {}", client_id, e);
            }
        }
    }
}
