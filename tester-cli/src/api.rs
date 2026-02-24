use anyhow::{bail, Result};
use subxt::{
    blocks::ExtrinsicEvents, error::DispatchError, tx::Payload, OnlineClient, PolkadotConfig,
};
use subxt_signer::sr25519::Keypair;

#[subxt::subxt(runtime_metadata_path = "artifacts/gaia.scale")]
pub mod gaia {}

pub type Client = OnlineClient<PolkadotConfig>;
pub type ByteBoundedVec = gaia::runtime_types::bounded_collections::bounded_vec::BoundedVec<u8>;

pub async fn connect(url: &str) -> Result<Client> {
    Ok(OnlineClient::<PolkadotConfig>::from_url(url).await?)
}

pub fn bounded_str(input: &str) -> ByteBoundedVec {
    gaia::runtime_types::bounded_collections::bounded_vec::BoundedVec(input.as_bytes().to_vec())
}

pub fn bounded_to_string(input: &ByteBoundedVec) -> String {
    String::from_utf8_lossy(&input.0).into_owned()
}

pub async fn submit_and_watch<Call>(
    client: &Client,
    payload: &Call,
    signer: &Keypair,
) -> Result<ExtrinsicEvents<PolkadotConfig>>
where
    Call: Payload,
{
    let submission = client
        .tx()
        .sign_and_submit_then_watch_default(payload, signer)
        .await;

    let tx_progress = match submission {
        Ok(progress) => progress,
        Err(error) => bail!("{}", format_subxt_error(&error)),
    };

    match tx_progress.wait_for_finalized_success().await {
        Ok(events) => Ok(events),
        Err(error) => bail!("{}", format_subxt_error(&error)),
    }
}

pub fn format_subxt_error(error: &subxt::Error) -> String {
    match error {
        subxt::Error::Runtime(dispatch_error) => {
            format!(
                "Runtime dispatch failed: {}",
                format_dispatch_error(dispatch_error)
            )
        }
        _ => error.to_string(),
    }
}

fn format_dispatch_error(error: &DispatchError) -> String {
    match error {
        DispatchError::Module(module_error) => match module_error.details() {
            Ok(details) => format!("{}::{}", details.pallet.name(), details.variant.name),
            Err(_) => module_error.to_string(),
        },
        _ => error.to_string(),
    }
}
