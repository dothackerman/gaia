use anyhow::{bail, Result};
use subxt::{
    blocks::ExtrinsicEvents, error::DispatchError, tx::Payload, OnlineClient, PolkadotConfig,
};
use subxt_signer::sr25519::Keypair;

#[subxt::subxt(runtime_metadata_path = "artifacts/gaia.scale")]
pub mod gaia {}

pub type Client = OnlineClient<PolkadotConfig>;
pub type ByteBoundedVec = gaia::runtime_types::bounded_collections::bounded_vec::BoundedVec<u8>;

const MAX_NAME_LEN: usize = 128;
const MAX_TITLE_LEN: usize = 128;
const MAX_DESCRIPTION_LEN: usize = 1024;

pub async fn connect(url: &str) -> Result<Client> {
    Ok(OnlineClient::<PolkadotConfig>::from_url(url).await?)
}

fn bounded_str(input: &str, max_len: usize, field: &str) -> Result<ByteBoundedVec> {
    if input.len() > max_len {
        bail!("{field} exceeds max length ({max_len} bytes)");
    }
    Ok(gaia::runtime_types::bounded_collections::bounded_vec::BoundedVec(
        input.as_bytes().to_vec(),
    ))
}

pub fn bounded_name(input: &str) -> Result<ByteBoundedVec> {
    bounded_str(input, MAX_NAME_LEN, "name")
}

pub fn bounded_title(input: &str) -> Result<ByteBoundedVec> {
    bounded_str(input, MAX_TITLE_LEN, "title")
}

pub fn bounded_description(input: &str) -> Result<ByteBoundedVec> {
    bounded_str(input, MAX_DESCRIPTION_LEN, "description")
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
