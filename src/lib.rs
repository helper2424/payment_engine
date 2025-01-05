pub mod transaction;
mod decimal;
pub mod account;
pub mod payment_engine;

use std::{error::Error, io::{Read, Write}};

use csv::{ReaderBuilder, WriterBuilder};
use payment_engine::PaymentEngine;
use transaction::TransactionEntity;


pub struct App {}

impl App {
    pub async fn run<R: Read, W: Write>(input: R, mut output: W, ordeded_output: bool) -> Result<(), Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(input);

        let mut engine = PaymentEngine::new();

        for result in reader.deserialize::<TransactionEntity>() {
            match result {
                Ok(transaction) => engine.process_transaction(transaction).await?,
                Err(err) => eprintln!("Error deserializing transaction: {}", err),
            }
        }

        engine.shutdown().await;

        let accounts = engine.get_account_entities(ordeded_output).await;
        
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(&mut output);

        for account in accounts {
            if let Err(err) = writer.serialize(account) {
                eprintln!("Error serializing account: {}", err);
            }
        }

        writer.flush()?;
        Ok(())
    }
}
