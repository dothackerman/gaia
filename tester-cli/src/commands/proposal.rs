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
        api::bounded_title(&title)?,
        api::bounded_description(&description)?,
        amount,
        event_block,
    );

    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if let Some(event) = events.find_first::<api::gaia::proposals::events::ProposalSubmitted>()? {
        println!(
            "Proposal submitted: id={}, organizer={}",
            event.proposal_id, event.organizer
        );
    } else {
        println!("Proposal submission finalized.");
    }

    println!("Finalized extrinsic hash: {}", events.extrinsic_hash());
    Ok(())
}

pub async fn vote(url: &str, signer: Persona, proposal_id: u32, approve: bool) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx()
        .proposals()
        .vote_on_proposal(proposal_id, approve);

    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if let Some(event) = events.find_first::<api::gaia::proposals::events::VoteCast>()? {
        println!(
            "Proposal vote recorded: proposal_id={}, voter={}, approve={}",
            event.proposal_id, event.voter, event.approve
        );
    } else {
        println!("Proposal vote finalized.");
    }

    println!(
        "Vote submitted by {} (extrinsic {}).",
        signer.label(),
        events.extrinsic_hash()
    );
    Ok(())
}

pub async fn tally(url: &str, signer: Persona, proposal_id: u32) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx().proposals().tally_proposal(proposal_id);

    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if events.has::<api::gaia::proposals::events::ProposalApproved>()? {
        println!("Tallied proposal {proposal_id}: Approved.");
    } else if events.has::<api::gaia::proposals::events::ProposalRejected>()? {
        println!("Tallied proposal {proposal_id}: Rejected.");
    } else {
        println!("Tallied proposal {proposal_id}.");
    }

    println!("Finalized extrinsic hash: {}", events.extrinsic_hash());
    Ok(())
}

pub async fn execute(url: &str, signer: Persona, proposal_id: u32) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx().proposals().execute_proposal(proposal_id);

    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if let Some(event) = events.find_first::<api::gaia::proposals::events::ProposalExecuted>()? {
        println!(
            "Proposal executed: id={}, organizer={}, amount={}",
            event.proposal_id, event.organizer, event.amount
        );
    } else {
        println!("Executed proposal {proposal_id}.");
    }

    println!("Finalized extrinsic hash: {}", events.extrinsic_hash());
    Ok(())
}
