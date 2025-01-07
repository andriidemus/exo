use anyhow::Result;
use exo::tui::start;

#[tokio::main]
pub async fn main() -> Result<()> {
    start()?;
    Ok(())
}
