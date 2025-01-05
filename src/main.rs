use std::fs::File;
use std::io;
use std::env;
use std::error::Error;
use payment_engine::App;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Usage: cargo run -- <transactions_file>".into());
    }
    
    let transactions_file = File::open(&args[1])?;
    let stdout = io::stdout();
    App::run(transactions_file, stdout, false).await?;
    Ok(())
}

