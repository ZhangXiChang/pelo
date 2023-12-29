mod app;
mod deck;
mod log_record;

use app::*;
use log_record::*;

#[tokio::main]
async fn main() {
    LogRecord::new().start();
    App::new().run().await;
}
