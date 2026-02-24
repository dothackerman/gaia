mod api;
mod commands;
mod personas;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{local, membership, persona, proposal, treasury, watch};
use personas::Persona;

#[derive(Parser, Debug)]
#[command(name = "gaia-tester", about = "GAIA local tester CLI")]
struct Cli {
    #[arg(long, default_value = "ws://127.0.0.1:9944")]
    url: String,

    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Subcommand, Debug)]
enum TopCommand {
    Persona {
        #[command(subcommand)]
        command: PersonaCommand,
    },
    Membership {
        #[command(subcommand)]
        command: MembershipCommand,
    },
    Proposal {
        #[command(subcommand)]
        command: ProposalCommand,
    },
    Treasury {
        #[command(subcommand)]
        command: TreasuryCommand,
    },
    Watch {
        #[command(subcommand)]
        command: WatchCommand,
    },
    Local {
        #[command(subcommand)]
        command: LocalCommand,
    },
}

#[derive(Subcommand, Debug)]
enum PersonaCommand {
    List,
    Preview { persona: Persona },
}

#[derive(Subcommand, Debug)]
enum MembershipCommand {
    Propose {
        signer: Persona,
        candidate: Persona,
    },
    Vote {
        signer: Persona,
        candidate: Persona,
        approve: VoteChoice,
    },
}

#[derive(Subcommand, Debug)]
enum ProposalCommand {
    Submit {
        signer: Persona,
        title: String,
        description: String,
        amount: u128,
        event_block: u32,
    },
    Vote {
        signer: Persona,
        proposal_id: u32,
        approve: VoteChoice,
    },
    Tally {
        signer: Persona,
        proposal_id: u32,
    },
    Execute {
        signer: Persona,
        proposal_id: u32,
    },
}

#[derive(Subcommand, Debug)]
enum TreasuryCommand {
    Deposit { signer: Persona, amount: u128 },
}

#[derive(Subcommand, Debug)]
enum WatchCommand {
    Proposal { proposal_id: u32 },
    Treasury,
}

#[derive(Subcommand, Debug)]
enum LocalCommand {
    Start,
    Reset,
    RefreshMetadata,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
enum VoteChoice {
    Yes,
    No,
}

impl VoteChoice {
    fn as_bool(self) -> bool {
        matches!(self, VoteChoice::Yes)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        TopCommand::Persona { command } => match command {
            PersonaCommand::List => persona::list(),
            PersonaCommand::Preview { persona } => persona::preview(persona),
        },
        TopCommand::Membership { command } => match command {
            MembershipCommand::Propose { signer, candidate } => {
                membership::propose_member(&cli.url, signer, candidate).await?
            }
            MembershipCommand::Vote {
                signer,
                candidate,
                approve,
            } => membership::vote_candidate(&cli.url, signer, candidate, approve.as_bool()).await?,
        },
        TopCommand::Proposal { command } => match command {
            ProposalCommand::Submit {
                signer,
                title,
                description,
                amount,
                event_block,
            } => {
                proposal::submit(&cli.url, signer, title, description, amount, event_block).await?
            }
            ProposalCommand::Vote {
                signer,
                proposal_id,
                approve,
            } => proposal::vote(&cli.url, signer, proposal_id, approve.as_bool()).await?,
            ProposalCommand::Tally {
                signer,
                proposal_id,
            } => proposal::tally(&cli.url, signer, proposal_id).await?,
            ProposalCommand::Execute {
                signer,
                proposal_id,
            } => proposal::execute(&cli.url, signer, proposal_id).await?,
        },
        TopCommand::Treasury { command } => match command {
            TreasuryCommand::Deposit { signer, amount } => {
                treasury::deposit(&cli.url, signer, amount).await?
            }
        },
        TopCommand::Watch { command } => match command {
            WatchCommand::Proposal { proposal_id } => {
                watch::proposal(&cli.url, proposal_id).await?
            }
            WatchCommand::Treasury => watch::treasury_balance(&cli.url).await?,
        },
        TopCommand::Local { command } => match command {
            LocalCommand::Start => local::print_start_node_hint(),
            LocalCommand::Reset => local::print_reset_hint(),
            LocalCommand::RefreshMetadata => local::print_metadata_hint(),
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_persona_preview_command() {
        let cli = Cli::try_parse_from(["gaia-tester", "persona", "preview", "alice"])
            .expect("persona preview should parse");
        assert!(matches!(
            cli.command,
            TopCommand::Persona {
                command: PersonaCommand::Preview {
                    persona: Persona::Alice
                }
            }
        ));
    }

    #[test]
    fn reject_unknown_persona() {
        let parsed = Cli::try_parse_from(["gaia-tester", "persona", "preview", "zoe"]);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_membership_vote_command() {
        let cli = Cli::try_parse_from([
            "gaia-tester",
            "membership",
            "vote",
            "alice",
            "charlie",
            "yes",
        ])
        .expect("membership vote should parse");
        assert!(matches!(
            cli.command,
            TopCommand::Membership {
                command: MembershipCommand::Vote {
                    signer: Persona::Alice,
                    candidate: Persona::Charlie,
                    approve: VoteChoice::Yes,
                }
            }
        ));
    }

    #[test]
    fn parse_proposal_submit_command() {
        let cli = Cli::try_parse_from([
            "gaia-tester",
            "proposal",
            "submit",
            "alice",
            "community-event",
            "fund-public-workshop",
            "500",
            "240",
        ])
        .expect("proposal submit should parse");
        assert!(matches!(
            cli.command,
            TopCommand::Proposal {
                command: ProposalCommand::Submit {
                    signer: Persona::Alice,
                    amount: 500,
                    event_block: 240,
                    ..
                }
            }
        ));
    }

    #[test]
    fn parse_local_refresh_metadata_command() {
        let cli = Cli::try_parse_from(["gaia-tester", "local", "refresh-metadata"])
            .expect("local refresh metadata hint should parse");
        assert!(matches!(
            cli.command,
            TopCommand::Local {
                command: LocalCommand::RefreshMetadata
            }
        ));
    }
}
