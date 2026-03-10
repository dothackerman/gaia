mod api;
mod commands;
mod output;
mod personas;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::{local, membership, persona, proposal, treasury, watch};
use personas::Persona;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "gaia-tester", about = "GAIA local tester CLI")]
struct Cli {
    #[arg(
        long,
        default_value = "ws://127.0.0.1:9944",
        help = "WebSocket endpoint used to connect to the GAIA node."
    )]
    url: String,

    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Subcommand, Debug)]
enum TopCommand {
    #[command(about = "Inspect seeded local personas.")]
    Personas {
        #[command(subcommand)]
        command: PersonaCommand,
    },
    #[command(about = "Submit, vote, or finalize membership proposals.")]
    Memberships {
        #[command(subcommand)]
        command: MembershipCommand,
    },
    #[command(about = "Run governance proposal lifecycle actions.")]
    Proposals {
        #[command(subcommand)]
        command: ProposalCommand,
    },
    #[command(about = "Deposit funds into the treasury.")]
    Treasury {
        #[command(subcommand)]
        command: TreasuryCommand,
    },
    #[command(about = "Read-only chain state inspection commands.")]
    Watch {
        #[command(subcommand)]
        command: WatchCommand,
    },
    #[command(about = "Local helper hints for node/tester workflows.")]
    Local {
        #[command(subcommand)]
        command: LocalCommand,
    },
}

#[derive(Subcommand, Debug)]
enum PersonaCommand {
    #[command(about = "List all seeded local personas.")]
    List,
    #[command(about = "Show the derived account address for one seeded persona (read-only).")]
    Preview {
        #[arg(help = "Seeded persona whose account address should be derived and displayed.")]
        persona: Persona,
    },
}

#[derive(Subcommand, Debug)]
enum MembershipCommand {
    #[command(about = "Submit a membership proposal for a candidate persona.")]
    Propose {
        #[arg(help = "Seeded persona whose key signs this membership proposal transaction.")]
        signer: Persona,
        #[arg(help = "Seeded persona being proposed as a new member.")]
        candidate: Persona,
    },
    #[command(about = "Cast a yes/no vote on a membership proposal.")]
    Vote {
        #[arg(help = "Seeded persona whose key signs this membership vote transaction.")]
        signer: Persona,
        #[arg(help = "Membership proposal id to vote on.")]
        proposal_id: u32,
        #[arg(help = "Vote choice for this proposal: yes or no.")]
        approve: VoteChoice,
    },
    #[command(about = "Finalize a membership proposal after its voting window ends.")]
    Finalize {
        #[arg(help = "Seeded persona whose key signs this membership finalize transaction.")]
        signer: Persona,
        #[arg(help = "Membership proposal id to finalize.")]
        proposal_id: u32,
    },
}

#[derive(Subcommand, Debug)]
enum ProposalCommand {
    #[command(about = "Submit a typed governance proposal.")]
    Submit {
        #[command(subcommand)]
        command: ProposalSubmitCommand,
    },
    #[command(about = "Upload a runtime code blob for a later constitutional upgrade proposal.")]
    UploadRuntimeCode {
        #[arg(help = "Seeded persona whose key signs this runtime-code upload transaction.")]
        signer: Persona,
        #[arg(help = "Path to the runtime Wasm blob to upload.")]
        code_path: PathBuf,
    },
    #[command(about = "Cast a yes/no vote on a proposal.")]
    Vote {
        #[arg(help = "Seeded persona whose key signs this proposal vote transaction.")]
        signer: Persona,
        #[arg(help = "Proposal id to vote on.")]
        proposal_id: u32,
        #[arg(help = "Vote choice for this proposal: yes or no.")]
        approve: VoteChoice,
    },
    #[command(about = "Finalize (tally) a proposal after voting ends.")]
    Finalize {
        #[arg(help = "Seeded persona whose key signs this proposal finalize transaction.")]
        signer: Persona,
        #[arg(help = "Proposal id to finalize.")]
        proposal_id: u32,
    },
    #[command(about = "Execute an approved proposal exactly once.")]
    Execute {
        #[arg(help = "Seeded persona whose key signs this proposal execution transaction.")]
        signer: Persona,
        #[arg(help = "Proposal id to execute.")]
        proposal_id: u32,
    },
}

#[derive(Subcommand, Debug)]
enum ProposalSubmitCommand {
    #[command(about = "Submit a standard treasury disbursement proposal.")]
    Disbursement {
        #[arg(help = "Seeded persona whose key signs this proposal submission transaction.")]
        signer: Persona,
        #[arg(help = "Short proposal title shown in watch output.")]
        title: String,
        #[arg(help = "Longer proposal description explaining the spending request.")]
        description: String,
        #[arg(help = "Seeded persona who should receive funds if approved.")]
        recipient: Persona,
        #[arg(help = "Requested treasury amount to disburse if approved.")]
        amount: u128,
    },
    #[command(about = "Submit a governance proposal to set the proposal voting period.")]
    SetProposalVotingPeriod {
        signer: Persona,
        title: String,
        description: String,
        #[arg(help = "New proposal voting period in blocks.")]
        blocks: u32,
    },
    #[command(about = "Submit a governance proposal to set the execution delay.")]
    SetExecutionDelay {
        signer: Persona,
        title: String,
        description: String,
        #[arg(help = "New execution delay in blocks.")]
        blocks: u32,
    },
    #[command(about = "Submit a governance proposal to set the membership voting period.")]
    SetMembershipVotingPeriod {
        signer: Persona,
        title: String,
        description: String,
        #[arg(help = "New membership voting period in blocks.")]
        blocks: u32,
    },
    #[command(about = "Submit a constitutional proposal to set the standard approval threshold.")]
    SetStandardThreshold {
        signer: Persona,
        title: String,
        description: String,
        numerator: u32,
        denominator: u32,
    },
    #[command(
        about = "Submit a constitutional proposal to set the governance approval threshold."
    )]
    SetGovernanceThreshold {
        signer: Persona,
        title: String,
        description: String,
        numerator: u32,
        denominator: u32,
    },
    #[command(
        about = "Submit a constitutional proposal to set the constitutional approval threshold."
    )]
    SetConstitutionalThreshold {
        signer: Persona,
        title: String,
        description: String,
        numerator: u32,
        denominator: u32,
    },
    #[command(
        about = "Submit a constitutional proposal to set the membership approval threshold."
    )]
    SetMembershipThreshold {
        signer: Persona,
        title: String,
        description: String,
        numerator: u32,
        denominator: u32,
    },
    #[command(about = "Submit a constitutional proposal to set the suspension threshold.")]
    SetSuspensionThreshold {
        signer: Persona,
        title: String,
        description: String,
        numerator: u32,
        denominator: u32,
    },
    #[command(
        about = "Submit a constitutional proposal to upgrade the runtime using a previously uploaded code blob."
    )]
    UpgradeRuntime {
        signer: Persona,
        title: String,
        description: String,
        #[arg(
            help = "Expected blake2-256 hash of the uploaded runtime code, as 0x-prefixed or plain hex."
        )]
        code_hash: String,
    },
}

#[derive(Subcommand, Debug)]
enum TreasuryCommand {
    #[command(about = "Deposit funds from a signer account into treasury.")]
    Deposit {
        #[arg(help = "Seeded persona whose key signs this treasury deposit transaction.")]
        signer: Persona,
        #[arg(help = "Amount to transfer into treasury.")]
        amount: u128,
    },
}

#[derive(Subcommand, Debug)]
enum WatchCommand {
    #[command(about = "Show one proposal by id, or list proposals when no id is given.")]
    Proposals {
        #[arg(help = "Optional proposal id for detail view.")]
        proposal_id: Option<u32>,
        #[arg(
            long,
            value_enum,
            default_value_t = watch::ProposalStateFilter::Active,
            help = "Filter proposal list by lifecycle state."
        )]
        state: watch::ProposalStateFilter,
        #[arg(
            long,
            value_enum,
            default_value_t = watch::ListOrder::Newest,
            help = "Sort order for list output."
        )]
        order: watch::ListOrder,
        #[arg(
            long,
            conflicts_with = "no_pager",
            help = "Force pager usage for list output even if stdout is not a TTY."
        )]
        pager: bool,
        #[arg(long, help = "Disable pager and print raw list output.")]
        no_pager: bool,
    },
    #[command(
        about = "Show one membership proposal by id, or list proposals when no id is given."
    )]
    Memberships {
        #[arg(help = "Optional membership proposal id for detail view.")]
        proposal_id: Option<u32>,
        #[arg(
            long,
            value_enum,
            default_value_t = watch::MembershipStateFilter::Active,
            help = "Filter membership proposal list by lifecycle state."
        )]
        state: watch::MembershipStateFilter,
        #[arg(
            long,
            value_enum,
            default_value_t = watch::ListOrder::Newest,
            help = "Sort order for list output."
        )]
        order: watch::ListOrder,
        #[arg(
            long,
            conflicts_with = "no_pager",
            help = "Force pager usage for list output even if stdout is not a TTY."
        )]
        pager: bool,
        #[arg(long, help = "Disable pager and print raw list output.")]
        no_pager: bool,
    },
    #[command(about = "Show current treasury balance.")]
    Treasury,
}

#[derive(Subcommand, Debug)]
enum LocalCommand {
    #[command(about = "Print command hint for starting a local fast-local node.")]
    Start,
    #[command(about = "Print command hint for resetting local temporary chain state.")]
    Reset,
    #[command(about = "Print command hint for refreshing tester CLI metadata artifact.")]
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
        TopCommand::Personas { command } => match command {
            PersonaCommand::List => persona::list(),
            PersonaCommand::Preview { persona } => persona::preview(persona),
        },
        TopCommand::Memberships { command } => match command {
            MembershipCommand::Propose { signer, candidate } => {
                membership::propose_member(&cli.url, signer, candidate).await?
            }
            MembershipCommand::Vote {
                signer,
                proposal_id,
                approve,
            } => {
                membership::vote_candidate(&cli.url, signer, proposal_id, approve.as_bool()).await?
            }
            MembershipCommand::Finalize {
                signer,
                proposal_id,
            } => membership::finalize(&cli.url, signer, proposal_id).await?,
        },
        TopCommand::Proposals { command } => match command {
            ProposalCommand::Submit { command } => match command {
                ProposalSubmitCommand::Disbursement {
                    signer,
                    title,
                    description,
                    recipient,
                    amount,
                } => {
                    proposal::submit_disbursement(
                        &cli.url,
                        signer,
                        title,
                        description,
                        recipient,
                        amount,
                    )
                    .await?
                }
                ProposalSubmitCommand::SetProposalVotingPeriod {
                    signer,
                    title,
                    description,
                    blocks,
                } => {
                    proposal::submit_set_proposal_voting_period(
                        &cli.url,
                        signer,
                        title,
                        description,
                        blocks,
                    )
                    .await?
                }
                ProposalSubmitCommand::SetExecutionDelay {
                    signer,
                    title,
                    description,
                    blocks,
                } => {
                    proposal::submit_set_execution_delay(
                        &cli.url,
                        signer,
                        title,
                        description,
                        blocks,
                    )
                    .await?
                }
                ProposalSubmitCommand::SetMembershipVotingPeriod {
                    signer,
                    title,
                    description,
                    blocks,
                } => {
                    proposal::submit_set_membership_voting_period(
                        &cli.url,
                        signer,
                        title,
                        description,
                        blocks,
                    )
                    .await?
                }
                ProposalSubmitCommand::SetStandardThreshold {
                    signer,
                    title,
                    description,
                    numerator,
                    denominator,
                } => {
                    proposal::submit_set_standard_threshold(
                        &cli.url,
                        signer,
                        title,
                        description,
                        numerator,
                        denominator,
                    )
                    .await?
                }
                ProposalSubmitCommand::SetGovernanceThreshold {
                    signer,
                    title,
                    description,
                    numerator,
                    denominator,
                } => {
                    proposal::submit_set_governance_threshold(
                        &cli.url,
                        signer,
                        title,
                        description,
                        numerator,
                        denominator,
                    )
                    .await?
                }
                ProposalSubmitCommand::SetConstitutionalThreshold {
                    signer,
                    title,
                    description,
                    numerator,
                    denominator,
                } => {
                    proposal::submit_set_constitutional_threshold(
                        &cli.url,
                        signer,
                        title,
                        description,
                        numerator,
                        denominator,
                    )
                    .await?
                }
                ProposalSubmitCommand::SetMembershipThreshold {
                    signer,
                    title,
                    description,
                    numerator,
                    denominator,
                } => {
                    proposal::submit_set_membership_threshold(
                        &cli.url,
                        signer,
                        title,
                        description,
                        numerator,
                        denominator,
                    )
                    .await?
                }
                ProposalSubmitCommand::SetSuspensionThreshold {
                    signer,
                    title,
                    description,
                    numerator,
                    denominator,
                } => {
                    proposal::submit_set_suspension_threshold(
                        &cli.url,
                        signer,
                        title,
                        description,
                        numerator,
                        denominator,
                    )
                    .await?
                }
                ProposalSubmitCommand::UpgradeRuntime {
                    signer,
                    title,
                    description,
                    code_hash,
                } => {
                    proposal::submit_upgrade_runtime(
                        &cli.url,
                        signer,
                        title,
                        description,
                        &code_hash,
                    )
                    .await?
                }
            },
            ProposalCommand::UploadRuntimeCode { signer, code_path } => {
                proposal::upload_runtime_code(&cli.url, signer, &code_path).await?
            }
            ProposalCommand::Vote {
                signer,
                proposal_id,
                approve,
            } => proposal::vote(&cli.url, signer, proposal_id, approve.as_bool()).await?,
            ProposalCommand::Finalize {
                signer,
                proposal_id,
            } => proposal::finalize(&cli.url, signer, proposal_id).await?,
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
            WatchCommand::Proposals {
                proposal_id,
                state,
                order,
                pager,
                no_pager,
            } => {
                watch::proposals(
                    &cli.url,
                    proposal_id,
                    watch::ProposalListOptions {
                        state,
                        order,
                        pager,
                        no_pager,
                    },
                )
                .await?
            }
            WatchCommand::Memberships {
                proposal_id,
                state,
                order,
                pager,
                no_pager,
            } => {
                watch::memberships(
                    &cli.url,
                    proposal_id,
                    watch::MembershipListOptions {
                        state,
                        order,
                        pager,
                        no_pager,
                    },
                )
                .await?
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
        let cli = Cli::try_parse_from(["gaia-tester", "personas", "preview", "alice"])
            .expect("persona preview should parse");
        assert!(matches!(
            cli.command,
            TopCommand::Personas {
                command: PersonaCommand::Preview {
                    persona: Persona::Alice
                }
            }
        ));
    }

    #[test]
    fn reject_unknown_persona() {
        let parsed = Cli::try_parse_from(["gaia-tester", "personas", "preview", "zoe"]);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_membership_vote_command() {
        let cli = Cli::try_parse_from(["gaia-tester", "memberships", "vote", "alice", "3", "yes"])
            .expect("membership vote should parse");
        assert!(matches!(
            cli.command,
            TopCommand::Memberships {
                command: MembershipCommand::Vote {
                    signer: Persona::Alice,
                    proposal_id: 3,
                    approve: VoteChoice::Yes,
                }
            }
        ));
    }

    #[test]
    fn parse_proposal_submit_command() {
        let cli = Cli::try_parse_from([
            "gaia-tester",
            "proposals",
            "submit",
            "disbursement",
            "alice",
            "community-event",
            "fund-public-workshop",
            "bob",
            "500",
        ])
        .expect("proposal submit should parse");
        assert!(matches!(
            cli.command,
            TopCommand::Proposals {
                command: ProposalCommand::Submit {
                    command: ProposalSubmitCommand::Disbursement {
                        signer: Persona::Alice,
                        recipient: Persona::Bob,
                        amount: 500,
                        ..
                    }
                }
            }
        ));
    }

    #[test]
    fn parse_runtime_upgrade_submit_command() {
        let cli = Cli::try_parse_from([
            "gaia-tester",
            "proposals",
            "submit",
            "upgrade-runtime",
            "alice",
            "upgrade",
            "apply-new-runtime",
            "0x0123",
        ])
        .expect("runtime-upgrade submit should parse");
        assert!(matches!(
            cli.command,
            TopCommand::Proposals {
                command: ProposalCommand::Submit {
                    command: ProposalSubmitCommand::UpgradeRuntime {
                        signer: Persona::Alice,
                        ..
                    }
                }
            }
        ));
    }

    #[test]
    fn parse_upload_runtime_code_command() {
        let cli = Cli::try_parse_from([
            "gaia-tester",
            "proposals",
            "upload-runtime-code",
            "alice",
            "runtime/target.wasm",
        ])
        .expect("runtime-code upload should parse");
        assert!(matches!(
            cli.command,
            TopCommand::Proposals {
                command: ProposalCommand::UploadRuntimeCode {
                    signer: Persona::Alice,
                    ..
                }
            }
        ));
    }

    #[test]
    fn parse_watch_proposals_list_command() {
        let cli = Cli::try_parse_from([
            "gaia-tester",
            "watch",
            "proposals",
            "--state",
            "all",
            "--order",
            "oldest",
            "--no-pager",
        ])
        .expect("watch proposals list should parse");

        assert!(matches!(
            cli.command,
            TopCommand::Watch {
                command: WatchCommand::Proposals {
                    proposal_id: None,
                    state: watch::ProposalStateFilter::All,
                    order: watch::ListOrder::Oldest,
                    pager: false,
                    no_pager: true,
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
