use crate::api;
use anyhow::Result;

pub async fn proposal(url: &str, proposal_id: u32) -> Result<()> {
    let client = api::connect(url).await?;
    let storage = api::gaia::storage().proposals().proposals(proposal_id);
    let maybe = client.storage().at_latest().await?.fetch(&storage).await?;

    match maybe {
        Some(record) => {
            let title = String::from_utf8_lossy(&record.title);
            println!(
                "Proposal #{proposal_id}: status={:?}, title={title}",
                record.status
            );
        }
        None => println!("Proposal #{proposal_id} not found."),
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
