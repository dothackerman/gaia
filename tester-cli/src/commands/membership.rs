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

    if let Some(event) =
        events.find_first::<api::gaia::membership::events::MemberProposalSubmitted>()?
    {
        println!(
            "Membership proposal submitted: id={}, candidate={}, proposer={}, vote_end={}",
            event.proposal_id, event.candidate, event.proposed_by, event.vote_end
        );
    } else {
        println!("Membership proposal submission finalized.");
    }

    if let Some(event) = events.find_first::<api::gaia::membership::events::MemberProposalApproved>()?
    {
        println!(
            "Membership proposal approved: id={}, member={}",
            event.proposal_id, event.member
        );
    }

    println!("Finalized extrinsic hash: {}", events.extrinsic_hash());
    Ok(())
}

pub async fn vote_candidate(
    url: &str,
    signer: Persona,
    proposal_id: u32,
    approve: bool,
) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx()
        .membership()
        .vote_on_candidate(proposal_id, approve);

    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if let Some(event) =
        events.find_first::<api::gaia::membership::events::MemberProposalVoteCast>()?
    {
        println!(
            "Membership vote recorded: proposal_id={}, candidate={}, voter={}, approve={}",
            event.proposal_id, event.candidate, event.voter, event.approve
        );
    } else {
        println!("Membership vote finalized.");
    }

    if let Some(event) = events.find_first::<api::gaia::membership::events::MemberProposalApproved>()?
    {
        println!(
            "Membership proposal approved: id={}, member={}",
            event.proposal_id, event.member
        );
    }

    println!(
        "Submitted memberships::vote by {} on proposal {} => {} (extrinsic {}).",
        signer.label(),
        proposal_id,
        approve,
        events.extrinsic_hash()
    );
    Ok(())
}

pub async fn finalize(url: &str, signer: Persona, proposal_id: u32) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx().membership().finalize_proposal(proposal_id);

    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if let Some(event) = events.find_first::<api::gaia::membership::events::MemberProposalApproved>()?
    {
        println!(
            "Membership proposal finalized: id={} approved for member={}",
            event.proposal_id, event.member
        );
    } else if let Some(event) =
        events.find_first::<api::gaia::membership::events::MemberProposalRejected>()?
    {
        println!(
            "Membership proposal finalized: id={} rejected for candidate={}",
            event.proposal_id, event.candidate
        );
    } else {
        println!("Membership proposal {proposal_id} finalized.");
    }

    println!("Finalized extrinsic hash: {}", events.extrinsic_hash());
    Ok(())
}
