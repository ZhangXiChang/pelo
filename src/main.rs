use anyhow::Result;

mod main_menu;
// mod side_menu;
mod system;

use main_menu::*;
use ratatui::layout::{Constraint, Direction, Layout};
// use side_menu::*;
use system::*;

#[tokio::main]
async fn main() -> Result<()> {
    System::nwe(SystemInfo {
        components: vec![Box::new(WidgetLayout {
            layout: Layout::new(Direction::Vertical, [Constraint::Min(0)]),
            widgets: vec![Widget::new(Box::new(MainMenu::new()), 0)],
            ..Default::default()
        })],
    })
    .run()?;
    Ok(())
}
