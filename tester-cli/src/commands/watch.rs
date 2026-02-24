use crate::api;
use anyhow::Result;

pub async fn proposal(url: &str, proposal_id: u32) -> Result<()> {
    let client = api::connect(url).await?;
    let storage = api::gaia::storage().proposals().proposals(proposal_id);
    let maybe = client.storage().at_latest().await?.fetch(&storage).await?;
    let yes_key = api::gaia::storage()
        .proposals()
        .proposal_yes_count(proposal_id);
    let no_key = api::gaia::storage()
        .proposals()
        .proposal_no_count(proposal_id);
    let yes_votes = client
        .storage()
        .at_latest()
        .await?
        .fetch_or_default(&yes_key)
        .await?;
    let no_votes = client
        .storage()
        .at_latest()
        .await?
        .fetch_or_default(&no_key)
        .await?;

    match maybe {
        Some(record) => {
            let title = api::bounded_to_string(&record.title);
            println!(
                "Proposal #{proposal_id}: status={:?}, title={title}, yes_votes={yes_votes}, no_votes={no_votes}, vote_end={}",
                record.status,
                record.vote_end
            );
        }
        None => println!(
            "Proposal #{proposal_id} not found (yes_votes={yes_votes}, no_votes={no_votes})."
        ),
    }
    Ok(())
}

pub async fn treasury_balance(url: &str) -> Result<()> {
    let client = api::connect(url).await?;
    let key = api::gaia::storage().treasury().treasury_balance();
    let balance = client
        .storage()
        .at_latest()
        .await?
        .fetch_or_default(&key)
        .await?;
    println!("Treasury balance: {balance}");
    Ok(())
}
