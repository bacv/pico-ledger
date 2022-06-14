# pico-ledger
Pico (very small) ledger implementation using rust

## How to run
The app can be started using cargo run:
```bash
cargo run -- txs.csv > acc.csv
```
Application uses stderr to print errors if they happen inside the app or repository layer.
CSV parsing error should make application panic.

## Assumptions that were made
Assuming that a chargeback can make the account negative.
Assuming that negative amount in a transaction is not allowed.
Assuming that withdrawal can't be disputed.

## Project's structure
Domain related structures and traits are defined in `dom/` folder. Ideally domain layer should not use any references from app and implementation layers.
Application related files are in `app` and implementation details are defined in `repo`.

Most of the logic is defined in [`src/repo/booking_repo.rs`](src/repo/booking_repo.rs)

## TODOs
* Add checks for Amount(i64) to f64 conversion;
* Use Tokio::fs::File and BufReader in the binary;
* Add proof of concept for a ledger that handles multiple transaction streams over the network;
