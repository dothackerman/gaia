use crate::{api, personas::Persona};
use anyhow::{bail, Context, Result};
use std::{fs, path::Path};
use subxt::utils::AccountId32;

type ProposalClass = api::gaia::runtime_types::gaia_proposals::pallet::ProposalClass;
type GovernanceAction =
    api::gaia::runtime_types::gaia_proposals::pallet::GovernanceAction<AccountId32, u128, u32>;

fn parse_code_hash(input: &str) -> Result<[u8; 32]> {
    let trimmed = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
        .unwrap_or(input);
    if trimmed.len() != 64 {
        bail!("code hash must contain exactly 32 bytes (64 hex chars)");
    }

    let mut out = [0u8; 32];
    for (i, chunk) in trimmed.as_bytes().chunks_exact(2).enumerate() {
        let pair = std::str::from_utf8(chunk).context("code hash must be valid ASCII hex")?;
        out[i] = u8::from_str_radix(pair, 16).context("code hash must be hex encoded")?;
    }
    Ok(out)
}

async fn submit(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    class: ProposalClass,
    action: GovernanceAction,
) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx().proposals().submit_proposal(
        api::bounded_title(&title)?,
        api::bounded_description(&description)?,
        class,
        action,
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

pub async fn submit_disbursement(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    recipient: Persona,
    amount: u128,
) -> Result<()> {
    let action = GovernanceAction::DisburseToAccount {
        recipient: recipient.account_id()?,
        amount,
    };
    submit(
        url,
        signer,
        title,
        description,
        ProposalClass::Standard,
        action,
    )
    .await
}

pub async fn submit_set_proposal_voting_period(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    blocks: u32,
) -> Result<()> {
    submit(
        url,
        signer,
        title,
        description,
        ProposalClass::Governance,
        GovernanceAction::SetProposalVotingPeriod { blocks },
    )
    .await
}

pub async fn submit_set_execution_delay(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    blocks: u32,
) -> Result<()> {
    submit(
        url,
        signer,
        title,
        description,
        ProposalClass::Governance,
        GovernanceAction::SetExecutionDelay { blocks },
    )
    .await
}

pub async fn submit_set_membership_voting_period(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    blocks: u32,
) -> Result<()> {
    submit(
        url,
        signer,
        title,
        description,
        ProposalClass::Governance,
        GovernanceAction::SetMembershipVotingPeriod { blocks },
    )
    .await
}

pub async fn submit_set_standard_threshold(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    numerator: u32,
    denominator: u32,
) -> Result<()> {
    submit(
        url,
        signer,
        title,
        description,
        ProposalClass::Constitutional,
        GovernanceAction::SetStandardApprovalThreshold {
            numerator,
            denominator,
        },
    )
    .await
}

pub async fn submit_set_governance_threshold(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    numerator: u32,
    denominator: u32,
) -> Result<()> {
    submit(
        url,
        signer,
        title,
        description,
        ProposalClass::Constitutional,
        GovernanceAction::SetGovernanceApprovalThreshold {
            numerator,
            denominator,
        },
    )
    .await
}

pub async fn submit_set_constitutional_threshold(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    numerator: u32,
    denominator: u32,
) -> Result<()> {
    submit(
        url,
        signer,
        title,
        description,
        ProposalClass::Constitutional,
        GovernanceAction::SetConstitutionalApprovalThreshold {
            numerator,
            denominator,
        },
    )
    .await
}

pub async fn submit_set_membership_threshold(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    numerator: u32,
    denominator: u32,
) -> Result<()> {
    submit(
        url,
        signer,
        title,
        description,
        ProposalClass::Constitutional,
        GovernanceAction::SetMembershipApprovalThreshold {
            numerator,
            denominator,
        },
    )
    .await
}

pub async fn submit_set_suspension_threshold(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    numerator: u32,
    denominator: u32,
) -> Result<()> {
    submit(
        url,
        signer,
        title,
        description,
        ProposalClass::Constitutional,
        GovernanceAction::SetSuspensionThreshold {
            numerator,
            denominator,
        },
    )
    .await
}

pub async fn submit_upgrade_runtime(
    url: &str,
    signer: Persona,
    title: String,
    description: String,
    code_hash: &str,
) -> Result<()> {
    submit(
        url,
        signer,
        title,
        description,
        ProposalClass::Constitutional,
        GovernanceAction::UpgradeRuntime {
            code_hash: parse_code_hash(code_hash)?,
        },
    )
    .await
}

pub async fn upload_runtime_code(url: &str, signer: Persona, code_path: &Path) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let code = fs::read(code_path)
        .with_context(|| format!("failed to read runtime code from {}", code_path.display()))?;
    let payload = api::gaia::tx().proposals().upload_runtime_code(code);
    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if let Some(event) = events.find_first::<api::gaia::proposals::events::RuntimeCodeUploaded>()? {
        println!(
            "Runtime code uploaded: uploader={}, code_hash=0x{}",
            event.uploader,
            format_hex(&event.code_hash)
        );
    } else {
        println!("Runtime code upload finalized.");
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

pub async fn finalize(url: &str, signer: Persona, proposal_id: u32) -> Result<()> {
    let client = api::connect(url).await?;
    let signer_key = signer.keypair()?;
    let payload = api::gaia::tx().proposals().tally_proposal(proposal_id);

    let events = api::submit_and_watch(&client, &payload, &signer_key).await?;

    if events.has::<api::gaia::proposals::events::ProposalApproved>()? {
        println!("Finalized proposal {proposal_id}: Approved.");
    } else if events.has::<api::gaia::proposals::events::ProposalRejected>()? {
        println!("Finalized proposal {proposal_id}: Rejected.");
    } else {
        println!("Finalized proposal {proposal_id}.");
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
        println!("Proposal executed: id={}", event.proposal_id);
    } else {
        println!("Executed proposal {proposal_id}.");
    }

    if let Some(event) =
        events.find_first::<api::gaia::proposals::events::RuntimeUpgradeExecuted>()?
    {
        println!(
            "Runtime upgrade executed with code_hash=0x{}",
            format_hex(&event.code_hash)
        );
    }

    println!("Finalized extrinsic hash: {}", events.extrinsic_hash());
    Ok(())
}

fn format_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
