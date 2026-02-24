use anyhow::Result;
use subxt::{OnlineClient, PolkadotConfig};

#[subxt::subxt(runtime_metadata_path = "artifacts/gaia.scale")]
pub mod gaia {}

pub type Client = OnlineClient<PolkadotConfig>;

pub async fn connect(url: &str) -> Result<Client> {
    Ok(OnlineClient::<PolkadotConfig>::from_url(url).await?)
}
