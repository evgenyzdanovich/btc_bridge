use bridge::btc_tracker::monitor_bridging_txns;
use bridge::data::BridgeTransaction;
use bridge::rollup_dispatcher::push_to_rollup;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

const QUEUE_SIZE: usize = 100_000;
const ROLLUP_BATCH_SIZE: usize = 10_000;

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = mpsc::channel(QUEUE_SIZE);

    // Spawn a task to fetch the data related to bridging transactions.
    tokio::spawn(async move {
        monitor_bridging_txns(sender).await;
    });

    // Spawn a task to relay information to the Rollup
    tokio::spawn(async move {
        let mut buffer: Vec<BridgeTransaction> = Vec::with_capacity(ROLLUP_BATCH_SIZE);
        while receiver.recv_many(&mut buffer, ROLLUP_BATCH_SIZE).await > 0 {
            push_to_rollup(&buffer).await;
        }
    });

    // Keep the main thread alive
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
