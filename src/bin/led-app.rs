use std::{sync::Arc, env, fs::File, io};

use futures::{lock::Mutex};
use pico_ledger::{app::Ledger, repo::{InMemoryAccountRepository, InMemoryBookingRepository}, dom::{Tx, BookingService, AccountService}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let account_repo = Arc::new(Mutex::new(InMemoryAccountRepository::new()));
    let booking_repo = Arc::new(Mutex::new(InMemoryBookingRepository::new(
        account_repo.clone()
    )));

    let ledger = Arc::new(Ledger::new(account_repo, booking_repo));

    let path = args[1].clone();
    let file = File::open(path)?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    for record in rdr.deserialize() {
        let r: Tx = record?;
        if let Err(e) = ledger.process_tx(r).await {
            eprintln!("Error while processing tx_id {} : {}", r.tx_id, e);
        };
    }

    let accounts = ledger.dump_accounts().await?;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .double_quote(true)
        .flexible(true)
        .from_writer(io::stdout());

    for a in accounts.iter() {
        wtr.serialize(a)?;
    }

    wtr.flush()?;
    Ok(())
}