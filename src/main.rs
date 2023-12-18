mod app;

use app::*;

fn main() {
    let mut app = App::new();
    while app.is_run() {
        app.draw();
    }
}
