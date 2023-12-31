mod app;
mod deck;
mod log_record;

use std::process::exit;

use app::*;
use log::error;
use log_record::*;

#[tokio::main]
async fn main() {
    match LogRecord::new()
        .log_level(log::LevelFilter::Debug)
        .log_mode(LogRecordMode::File)
        .log_file_path("./logs/latest.log".to_string())
        .start()
    {
        Ok(_) => (),
        Err(e) => {
            panic!("{}", e);
        }
    }
    match App::new().run().await {
        Ok(_) => (),
        Err(e) => {
            error!("{}", e);
            exit(-1);
        }
    }
}
