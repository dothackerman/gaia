use anyhow::{Context, Result};
use std::path::PathBuf;
use subxt::ext::codec::Encode;
use subxt::{OnlineClient, PolkadotConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let url = args
        .next()
        .unwrap_or_else(|| "ws://127.0.0.1:9944".to_string());
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tester-cli/artifacts/gaia.scale"));

    let client = OnlineClient::<PolkadotConfig>::from_url(url.clone())
        .await
        .with_context(|| format!("failed to connect to {url}"))?;

    let metadata = client.metadata().encode();

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory {}", parent.display()))?;
    }

    std::fs::write(&output, metadata)
        .with_context(|| format!("failed to write metadata file {}", output.display()))?;

    println!("Wrote metadata to {}", output.display());
    Ok(())
}
