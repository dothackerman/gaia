use crate::{api, personas::Persona};
use anyhow::Result;

pub async fn deposit(url: &str, signer: Persona, amount: u128) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx().treasury().deposit_fee(amount);

    client
        .tx()
        .sign_and_submit_then_watch_default(&payload, &signer_key)
        .await?
        .wait_for_finalized_success()
        .await?;

    println!("Deposited {amount} to treasury as {}.", signer.label());
    Ok(())
}
