# Payment Engine

A toy payment processing engine built in Rust that handles client accounts and basic transaction types. This project demonstrates transaction processing concepts using a worker-per-account model.

## Features

- Basic transaction processing using Tokio
- Account handling with dedicated workers
- Support for transaction types:
  - Deposits
  - Withdrawals
  - Disputes
  - Resolves
  - Chargebacks
- Decimal precision handling (4 decimal places)
- CSV input/output
- Basic error handling

## Quick Start

```bash
cargo run -- transactions.csv > accounts.csv
```

## Input Format

The input CSV file should contain transactions in the following format:

```csv
type,client,tx,amount
deposit,1,1,1.0
withdrawal,1,2,1.5
dispute,1,1,
resolve,1,1,
chargeback,1,1,
```

Fields:
- `type`: Transaction type (deposit, withdrawal, dispute, resolve, chargeback)
- `client`: Client ID (u16)
- `tx`: Transaction ID (u32)
- `amount`: Transaction amount (decimal, optional for disputes/resolves/chargebacks)

## Output Format

The output is a CSV file containing the final state of all client accounts:

```csv
client,available,held,total,locked
1,1.5,0.0,1.5,false
2,2.0,0.0,2.0,false
```

Fields:
- `client`: Client ID
- `available`: Available funds
- `held`: Held funds (disputed)
- `total`: Total funds (available + held)
- `locked`: Account lock status

## Tests

Run tests to check that the engine works as expected.
```bash
cargo test
```

## Architecture

- **Actor Model**: Each account has a dedicated worker for transaction processing
- **Async Processing**: Built on Tokio for efficient async I/O and task management
- **Thread Safety**: Uses Mutex and Arc for safe concurrent access
- **Error Handling**: Comprehensive error handling for all transaction types

I wasn't sure about the exact requirements related to the software usage, so decided the way where created one tokio handler per account. Depends on the usage it could be optimized with several preinitialized handlers which handle multiple accounts. For example we can create 4 threads and distribute accounts per threads based on the hash of the client id (like `client_id % 4` if we have 4 threads).

## Transaction Rules

1. **Deposits**: Add funds to available balance, if the account is not locked
2. **Withdrawals**: Remove funds if sufficient balance exists, if the account is not locked and there are enough funds
3. **Disputes**: Hold funds from a previous transaction, if the account is not locked, the transaction is not disputed yet and there are enough funds to dispute
4. **Resolves**: Release held funds back to available, if the account is not locked, the transaction is disputed
5. **Chargebacks**: Reverse a transaction and lock the account, if the account is not locked, the transaction is disputed

## Error Handling

The engine handles various error cases:
- Insufficient funds for withdrawals
- Invalid transaction amounts
- Disputes on non-existent transactions
- Multiple disputes on same transaction
- Operations on locked accounts
