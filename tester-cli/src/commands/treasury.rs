use crate::{api, personas::Persona};
use anyhow::Result;

pub async fn deposit(url: &str, signer: Persona, amount: u128) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx().treasury().deposit_fee(amount);

    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if let Some(event) = events.find_first::<api::gaia::treasury::events::FeeDeposited>()? {
        println!(
            "Treasury fee deposited: from={}, amount={}, new_balance={}",
            event.from, event.amount, event.new_balance
        );
    } else {
        println!("Deposited {amount} to treasury as {}.", signer.label());
    }

    println!("Finalized extrinsic hash: {}", events.extrinsic_hash());
    Ok(())
}
