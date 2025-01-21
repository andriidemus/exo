use anyhow::Result;
use exo::tui::start;
use std::env;
use std::fs::read_to_string;

#[tokio::main]
pub async fn main() -> Result<()> {
    let preload = env::args()
        .nth(1)
        .and_then(|file| read_to_string(file).ok());

    start(preload).await?;
    Ok(())
}
