use crate::{api, output};
use anyhow::Result;

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListOrder {
    Newest,
    Oldest,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStateFilter {
    Active,
    Approved,
    Rejected,
    Executed,
    All,
}

impl ProposalStateFilter {
    fn matches_status(self, status: &str) -> bool {
        match self {
            ProposalStateFilter::Active => status == "active",
            ProposalStateFilter::Approved => status == "approved",
            ProposalStateFilter::Rejected => status == "rejected",
            ProposalStateFilter::Executed => status == "executed",
            ProposalStateFilter::All => true,
        }
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembershipStateFilter {
    Active,
    Approved,
    Rejected,
    All,
}

impl MembershipStateFilter {
    fn matches_status(self, status: &str) -> bool {
        match self {
            MembershipStateFilter::Active => status == "active",
            MembershipStateFilter::Approved => status == "approved",
            MembershipStateFilter::Rejected => status == "rejected",
            MembershipStateFilter::All => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProposalListOptions {
    pub state: ProposalStateFilter,
    pub order: ListOrder,
    pub pager: bool,
    pub no_pager: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MembershipListOptions {
    pub state: MembershipStateFilter,
    pub order: ListOrder,
    pub pager: bool,
    pub no_pager: bool,
}

pub async fn proposals(
    url: &str,
    proposal_id: Option<u32>,
    options: ProposalListOptions,
) -> Result<()> {
    if let Some(id) = proposal_id {
        return proposal(url, id).await;
    }

    let client = api::connect(url).await?;
    let at = client.storage().at_latest().await?;
    let count = at
        .fetch_or_default(&api::gaia::storage().proposals().proposal_count())
        .await?;

    let mut out = format!(
        "Treasury proposals (state={:?}, order={:?})\n",
        options.state, options.order
    );

    if count == 0 {
        out.push_str("No treasury proposals found.\n");
        output::print_or_page(&out, options.pager, options.no_pager)?;
        return Ok(());
    }

    let mut shown = 0usize;

    for id in ordered_ids(count, options.order) {
        let key = api::gaia::storage().proposals().proposals(id);
        let Some(record) = at.fetch(&key).await? else {
            continue;
        };

        let status_raw = format!("{:?}", record.status);
        let status = status_raw.to_lowercase();
        if !options.state.matches_status(&status) {
            continue;
        }

        let title = api::bounded_to_string(&record.title);
        let yes_votes = at
            .fetch_or_default(&api::gaia::storage().proposals().proposal_yes_count(id))
            .await?;
        let no_votes = at
            .fetch_or_default(&api::gaia::storage().proposals().proposal_no_count(id))
            .await?;

        out.push_str(&format!(
            "#{} status={} title={} amount={} organizer={} yes_votes={} no_votes={} vote_end={}\n",
            id,
            status,
            title,
            record.amount,
            record.organizer,
            yes_votes,
            no_votes,
            record.vote_end,
        ));
        shown = shown.saturating_add(1);
    }

    if shown == 0 {
        out.push_str("No treasury proposals matched this filter.\n");
    }

    output::print_or_page(&out, options.pager, options.no_pager)?;
    Ok(())
}

pub async fn proposal(url: &str, proposal_id: u32) -> Result<()> {
    let client = api::connect(url).await?;
    let at = client.storage().at_latest().await?;
    let storage = api::gaia::storage().proposals().proposals(proposal_id);
    let maybe = at.fetch(&storage).await?;
    let yes_key = api::gaia::storage()
        .proposals()
        .proposal_yes_count(proposal_id);
    let no_key = api::gaia::storage()
        .proposals()
        .proposal_no_count(proposal_id);
    let yes_votes = at.fetch_or_default(&yes_key).await?;
    let no_votes = at.fetch_or_default(&no_key).await?;

    match maybe {
        Some(record) => {
            let title = api::bounded_to_string(&record.title);
            let description = api::bounded_to_string(&record.description);
            println!(
                "Proposal #{}: status={:?}, title={}, description={}, amount={}, organizer={}, yes_votes={}, no_votes={}, event_block={}, vote_end={}",
                proposal_id,
                record.status,
                title,
                description,
                record.amount,
                record.organizer,
                yes_votes,
                no_votes,
                record.event_block,
                record.vote_end,
            );
        }
        None => println!(
            "Proposal #{} not found (yes_votes={}, no_votes={}).",
            proposal_id, yes_votes, no_votes
        ),
    }
    Ok(())
}

pub async fn memberships(
    url: &str,
    proposal_id: Option<u32>,
    options: MembershipListOptions,
) -> Result<()> {
    if let Some(id) = proposal_id {
        return membership(url, id).await;
    }

    let client = api::connect(url).await?;
    let at = client.storage().at_latest().await?;
    let count = at
        .fetch_or_default(&api::gaia::storage().membership().membership_proposal_count())
        .await?;

    let mut out = format!(
        "Membership proposals (state={:?}, order={:?})\n",
        options.state, options.order
    );

    if count == 0 {
        out.push_str("No membership proposals found.\n");
        output::print_or_page(&out, options.pager, options.no_pager)?;
        return Ok(());
    }

    let mut shown = 0usize;

    for id in ordered_ids(count, options.order) {
        let key = api::gaia::storage().membership().membership_proposals(id);
        let Some(record) = at.fetch(&key).await? else {
            continue;
        };

        let status_raw = format!("{:?}", record.status);
        let status = status_raw.to_lowercase();
        if !options.state.matches_status(&status) {
            continue;
        }

        let candidate_name = api::bounded_to_string(&record.name);
        let yes_votes = at
            .fetch_or_default(&api::gaia::storage().membership().membership_proposal_yes_count(id))
            .await?;
        let no_votes = at
            .fetch_or_default(&api::gaia::storage().membership().membership_proposal_no_count(id))
            .await?;

        out.push_str(&format!(
            "#{} status={} candidate={} name={} proposed_by={} yes_votes={} no_votes={} vote_end={}\n",
            id,
            status,
            record.candidate,
            candidate_name,
            record.proposed_by,
            yes_votes,
            no_votes,
            record.vote_end,
        ));
        shown = shown.saturating_add(1);
    }

    if shown == 0 {
        out.push_str("No membership proposals matched this filter.\n");
    }

    output::print_or_page(&out, options.pager, options.no_pager)?;
    Ok(())
}

pub async fn membership(url: &str, proposal_id: u32) -> Result<()> {
    let client = api::connect(url).await?;
    let at = client.storage().at_latest().await?;

    let key = api::gaia::storage()
        .membership()
        .membership_proposals(proposal_id);
    let maybe = at.fetch(&key).await?;

    let yes_votes = at
        .fetch_or_default(&api::gaia::storage().membership().membership_proposal_yes_count(proposal_id))
        .await?;
    let no_votes = at
        .fetch_or_default(&api::gaia::storage().membership().membership_proposal_no_count(proposal_id))
        .await?;

    match maybe {
        Some(record) => {
            let candidate_name = api::bounded_to_string(&record.name);
            println!(
                "Membership proposal #{}: status={:?}, candidate={}, name={}, proposed_by={}, proposed_at={}, vote_end={}, active_member_snapshot={}, yes_votes={}, no_votes={}",
                proposal_id,
                record.status,
                record.candidate,
                candidate_name,
                record.proposed_by,
                record.proposed_at,
                record.vote_end,
                record.active_member_snapshot,
                yes_votes,
                no_votes,
            );
        }
        None => {
            println!(
                "Membership proposal #{} not found (yes_votes={}, no_votes={}).",
                proposal_id, yes_votes, no_votes
            );
        }
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

fn ordered_ids(count: u32, order: ListOrder) -> Box<dyn Iterator<Item = u32>> {
    match order {
        ListOrder::Newest => Box::new((1..=count).rev()),
        ListOrder::Oldest => Box::new(1..=count),
    }
}
