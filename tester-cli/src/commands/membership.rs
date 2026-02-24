use crate::{api, personas::Persona};
use anyhow::Result;

pub async fn propose_member(url: &str, signer: Persona, candidate: Persona) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let candidate_id = candidate.account_id()?;
    let payload = api::gaia::tx()
        .membership()
        .propose_member(candidate_id.into());

    let events = client
        .tx()
        .sign_and_submit_then_watch_default(&payload, &signer_key)
        .await?
        .wait_for_finalized_success()
        .await?;

    println!(
        "Submitted membership::propose_member by {} for {} in block {}",
        signer.label(),
        candidate.label(),
        events.block_hash()
    );
    Ok(())
}

pub async fn vote_candidate(
    url: &str,
    signer: Persona,
    candidate: Persona,
    approve: bool,
) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx()
        .membership()
        .vote_on_candidate(candidate.account_id()?.into(), approve);

    client
        .tx()
        .sign_and_submit_then_watch_default(&payload, &signer_key)
        .await?
        .wait_for_finalized_success()
        .await?;

    println!(
        "Submitted membership::vote_on_candidate by {} for {} => {}",
        signer.label(),
        candidate.label(),
        approve
    );
    Ok(())
}
