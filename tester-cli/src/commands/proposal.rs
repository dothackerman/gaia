use crate::{api, personas::Persona};
use anyhow::Result;

pub async fn submit(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    amount: u128,
    event_block: u32,
) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx().proposals().submit_proposal(
        title.into_bytes(),
        description.into_bytes(),
        amount,
        event_block,
    );

    client
        .tx()
        .sign_and_submit_then_watch_default(&payload, &signer_key)
        .await?
        .wait_for_finalized_success()
        .await?;

    println!("Submitted proposal as {}.", signer.label());
    Ok(())
}

pub async fn vote(url: &str, signer: Persona, proposal_id: u32, approve: bool) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx()
        .proposals()
        .vote_on_proposal(proposal_id, approve);

    client
        .tx()
        .sign_and_submit_then_watch_default(&payload, &signer_key)
        .await?
        .wait_for_finalized_success()
        .await?;

    println!("Vote submitted by {}.", signer.label());
    Ok(())
}

pub async fn tally(url: &str, signer: Persona, proposal_id: u32) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx().proposals().tally_proposal(proposal_id);

    client
        .tx()
        .sign_and_submit_then_watch_default(&payload, &signer_key)
        .await?
        .wait_for_finalized_success()
        .await?;

    println!("Tallied proposal {proposal_id}.");
    Ok(())
}

pub async fn execute(url: &str, signer: Persona, proposal_id: u32) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx().proposals().execute_proposal(proposal_id);

    client
        .tx()
        .sign_and_submit_then_watch_default(&payload, &signer_key)
        .await?
        .wait_for_finalized_success()
        .await?;

    println!("Executed proposal {proposal_id}.");
    Ok(())
}
