use anyhow::Result;

mod main_menu;
mod side_menu;
mod system;
mod title;

use main_menu::*;
use ratatui::layout::{Constraint, Direction, Layout};
use side_menu::*;
use system::*;
use title::*;

#[tokio::main]
async fn main() -> Result<()> {
    System::nwe(SystemInfo {
        components: vec![Box::new(WidgetLayout {
            layout: Layout::new(
                Direction::Vertical,
                [Constraint::Length(3), Constraint::Min(0)],
            ),
            widgets: vec![Widget::new(Box::new(Title::new()), 0)],
            sub_layout: vec![Box::new(WidgetLayout {
                layout: Layout::new(
                    Direction::Horizontal,
                    [Constraint::Length(21), Constraint::Min(0)],
                ),
                widgets: vec![
                    Widget::new(Box::new(MainMenu::new()), 0),
                    Widget::new(Box::new(SideMenu::new()), 1),
                ],
                super_layout_area_index: 1,
                ..Default::default()
            })],
            ..Default::default()
        })],
    })
    .run()?;
    Ok(())
}
