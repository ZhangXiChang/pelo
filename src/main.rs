mod app;
mod log_record;

use app::*;
use log_record::*;

fn main() {
    LogRecord::new().enable();
    App::new().run();
}
