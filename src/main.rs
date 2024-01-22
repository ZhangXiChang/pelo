use anyhow::Result;

mod system;
use system::*;

#[tokio::main]
async fn main() -> Result<()> {
    System::nwe(SystemInfo::default()).run()?;
    Ok(())
}
