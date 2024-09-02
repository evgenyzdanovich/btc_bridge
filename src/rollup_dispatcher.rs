use crate::block_info::write_block_info;
use crate::data::BridgeTransaction;
use reqwest::Client;

const ROLLUP_NODE_ENDPOINT: &str = "https://rollup.mainnet.alpenlabs.io";

// Function to relay the transaction to Layer-2
pub async fn push_to_rollup(bridge_txns: &Vec<BridgeTransaction>) {
    let client = Client::new();
    let txns_serialized = bincode::serialize(bridge_txns).unwrap();

    match client
        .post(ROLLUP_NODE_ENDPOINT)
        .body(txns_serialized)
        .send()
        .await
    {
        Ok(response) => {
            println!(
                "Transactions are relayed: {:?}, count: {:?}",
                response.text().await.unwrap(),
                bridge_txns.len()
            );
            // Determine the block for which all the transaction are sent to rollup.
            // TODO: fix block hash.
            let block_hash = "FAKE_BLOCK_HASH";
            write_block_info((get_latest_handled_block(bridge_txns), block_hash))
        }
        Err(e) => {
            println!("Failed to relay transaction: {:?}", e);
        }
    }
}

fn get_latest_handled_block(bridge_txns: &Vec<BridgeTransaction>) -> u32 {
    let (mut min_block, mut max_block) = (0, 0);
    for txn in bridge_txns.iter() {
        min_block = std::cmp::min(min_block, txn.block_number);
        max_block = std::cmp::max(max_block, txn.block_number);
    }
    if max_block > min_block {
        // max_block - 1 is completely handled.
        max_block - 1
    } else {
        // we can't guarantee that anything after min_block - 1 is handled.
        min_block - 1
    }
}
