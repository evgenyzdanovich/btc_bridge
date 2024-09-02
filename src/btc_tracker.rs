use crate::block_info::read_block_info;
use crate::data::BridgeTransaction;
use bitcoin::consensus::deserialize;
use bitcoin::{Transaction, TxOut};
use hex;
use reqwest::Client;
use serde::Deserialize;
use serde_json;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

const BRIDGE_BTC_ADDRESS: &str = "bc1AlpenLabsBTCAddress";
const BLOCKSTREAM_TXN_PAGE_SIZE: usize = 25;

// Fetches the latest block fully handled by the bridge (rollup accepted the full block).
fn get_latest_handled_block_number() -> u32 {
    let block_info = read_block_info();
    if block_info.is_some() {
        return block_info.unwrap().0;
    }
    // if no data is found in the storage, we return the block number at which
    // the deployment of BRIDGE_BTC_ADDRESS is happened.
    // A hardcoded constant for the purpose of testing.
    850_000u32
}

// Auxiliary struct to get the latest block info.
#[derive(Deserialize)]
struct BlockMetaInfo {
    pub id: String,
    pub height: u32,
}

async fn get_block_hash(client: &Client, block_number: u32) -> Option<String> {
    match client
        .get(format!(
            "https://blockstream.info/api/block-height/{}",
            block_number
        ))
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(body) = response.text().await {
                Some(body)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

async fn get_transactions(client: &Client, block_hash: String) -> Vec<Transaction> {
    let mut v: Vec<Transaction> = vec![];
    let mut start_page_idx: usize = 0;

    loop {
        match client
            .get(format!(
                "https://blockstream.info/api/block-height/block/{}/txs/{}",
                block_hash, start_page_idx
            ))
            .send()
            .await
        {
            Ok(response) => {
                let mut tx_count: usize = 0;
                if let Ok(body) = response.text().await {
                    let cur_page: Vec<String> = serde_json::from_str(&body).unwrap();
                    tx_count = cur_page.len();
                    for tx_hex in cur_page {
                        let tx_bytes = hex::decode(tx_hex).unwrap();
                        let txn: Transaction = deserialize(&tx_bytes).unwrap();
                        v.push(txn);
                    }
                }
                // Update the index so we can fetch the next chunk of txns.
                start_page_idx += tx_count;
                if tx_count < BLOCKSTREAM_TXN_PAGE_SIZE {
                    break;
                }
            }
            Err(e) => {
                println!("Error fetching transactions: {:?}, retrying", e);
                sleep(Duration::from_secs(5)).await;
                // TODO: we shall retry fetching transactions asyncronously with some finite policy.
                // For now, the policy is very simple: wait for a bit and retry...
                // N.B. We didn't update `start_page_idx` so the next iteration attempts to make it.
            }
        };
    }
    v
}

fn is_output_belongs_to_bridge_address(_tx_out: &TxOut) -> bool {
    // TODO: Check UTXO if it's unlockable by the `BTC_BRIDGE_ADDRESS`.
    true
}

fn filter_bridge_transactions(btc_txns: &Vec<Transaction>) -> Vec<BridgeTransaction> {
    btc_txns
        .iter()
        .filter(|btc_txn| {
            btc_txn
                .output
                .iter()
                .any(is_output_belongs_to_bridge_address)
        })
        .map(|txn| txn.into())
        .collect()
}

// Monitors the BTC blockchain by fetching transactions block by block and populates the data into the channel.
pub async fn monitor_bridging_txns(channel: mpsc::Sender<BridgeTransaction>) {
    // N.B. /address/:address API is not quite suitable for our purposes because we need to handle
    // transactions from oldest to newest.
    // So we fetch block by block starting from the latest handled block, filter transactions related
    // to the `BRIDGE_BTC_ADDRESS` and put in into the channel's queue.
    loop {
        // For simplicity, we can use blockstream API.
        // In production, we might as well consider running our own node.
        let client = Client::new();
        let cur_block_number = get_latest_handled_block_number() + 1;

        // Fetch the height of the latest "confirmed" block.
        let mut latest_block_number: u32 = 0;
        match client
            .get("https://blockstream.info/api/blocks/tip")
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(body) = response.text().await {
                    // TODO handle parsing gracefully...
                    let latest_blocks: Vec<BlockMetaInfo> = serde_json::from_str(&body).unwrap();
                    // Although it's rare, let's be a bit more fork-proof
                    // and get the height of the block iwth 1 confirmation.
                    latest_block_number = latest_blocks[1].height;
                }
            }
            Err(e) => {
                println!("Error fetching latest block info: {:?}", e);
            }
        }
        // Check if we have a new block.
        // If not, just fallback to sleep in the loop.
        if latest_block_number > cur_block_number {
            let block_hash = get_block_hash(&client, cur_block_number).await;

            if let Some(hash) = block_hash {
                // TODO handle get_transactions gracefully...
                let txns = get_transactions(&client, hash).await;
                let bridge_txns = filter_bridge_transactions(&txns);
                for txn in bridge_txns {
                    channel.send(txn).await.unwrap();
                }
            }
        }
        // Polling for the purpose of testing.
        // TODO: In production, we can get rid of polling and use streaming or a webhook to receive data from our own node.
        sleep(Duration::from_secs(10)).await;
    }
}
