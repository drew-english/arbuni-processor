mod explorer;
mod models;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

const USDC_ADDRESS: &str = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";

fn init_logger() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[tokio::main]
async fn main() {
    init_logger();
    dotenv::dotenv().unwrap();

    explorer::find_and_update_all_pools(USDC_ADDRESS.to_string()).await;
}
