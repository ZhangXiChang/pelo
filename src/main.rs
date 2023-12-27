mod app;

use app::*;

fn main() {
    Log::new().enable();
    App::new().run();
}
