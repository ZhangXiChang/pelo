use anyhow::Result;

mod main_menu;
mod side_menu;
mod system;

use main_menu::*;
use side_menu::*;
use system::*;

#[tokio::main]
async fn main() -> Result<()> {
    System::nwe(SystemInfo {
        system_components: vec![Box::new(MainMenu::new()), Box::new(SideMenu::new())],
    })
    .run()?;
    Ok(())
}
