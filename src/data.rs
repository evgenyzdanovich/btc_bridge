use bitcoin::Transaction;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TransactionType {
    Withdrawal,
    Deposit,
}

// A type that represents bits of the "bridging" transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BridgeTransaction {
    pub txid: String,
    pub amount: u64,
    pub transaction_type: TransactionType,
    pub block_number: u32,
}

impl From<&Transaction> for BridgeTransaction {
    fn from(_btc_txn: &Transaction) -> Self {
        // TODO: mocked fn.
        // WE should contruct BridgeTransaction from BTC Transaction here.
        BridgeTransaction {
            txid: "".to_string(),
            amount: 0,
            transaction_type: TransactionType::Withdrawal,
            block_number: 850_000,
        }
    }
}
