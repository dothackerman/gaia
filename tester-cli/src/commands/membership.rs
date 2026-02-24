use crate::{api, personas::Persona};
use anyhow::Result;

pub async fn propose_member(url: &str, signer: Persona, candidate: Persona) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let candidate_id = candidate.account_id()?;
    let payload = api::gaia::tx()
        .membership()
        .propose_member(candidate_id, api::bounded_name(candidate.label())?);

    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if let Some(event) = events.find_first::<api::gaia::membership::events::CandidateProposed>()? {
        println!(
            "Membership candidate proposed: candidate={}, proposer={}",
            event.candidate, event.proposed_by
        );
    } else {
        println!("Membership candidate proposal finalized.");
    }

    if let Some(event) = events.find_first::<api::gaia::membership::events::MemberApproved>()? {
        println!(
            "Candidate immediately approved as active member: {}",
            event.member
        );
    }

    println!("Finalized extrinsic hash: {}", events.extrinsic_hash());
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
        .vote_on_candidate(candidate.account_id()?, approve);

    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if let Some(event) = events.find_first::<api::gaia::membership::events::VoteCast>()? {
        println!(
            "Membership vote recorded: voter={}, candidate={}, approve={}",
            event.voter, event.candidate, event.approve
        );
    } else {
        println!("Membership vote finalized.");
    }

    if let Some(event) = events.find_first::<api::gaia::membership::events::MemberApproved>()? {
        println!("Candidate approved as active member: {}", event.member);
    }

    println!(
        "Submitted membership::vote_on_candidate by {} for {} => {} (extrinsic {}).",
        signer.label(),
        candidate.label(),
        approve,
        events.extrinsic_hash()
    );
    Ok(())
}
