use std::process::ExitCode;

use clap::{Parser, Subcommand};

use gemini_marketplace::marketplace::commands::list::{execute as execute_list, ListOptions};

#[derive(Debug, Parser)]
#[command(
    name = "gemini-marketplace",
    version,
    about = "Gemini CLI extension marketplace",
    propagate_version = true
)]
struct MarketplaceCli {
    #[command(subcommand)]
    command: MarketplaceCommand,
}

#[derive(Debug, Subcommand)]
enum MarketplaceCommand {
    /// List available extensions
    List {
        #[arg(short, long)]
        search: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        installed: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show detailed information about an extension
    Show {
        #[arg(value_name = "SOURCE/EXTENSION")]
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Manage marketplace sources
    Sources {
        #[command(subcommand)]
        subcommand: SourcesCommand,
    },
    /// Refresh marketplace cache
    Refresh {
        #[arg(long)]
        force: bool,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Display overall status
    Status {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum SourcesCommand {
    /// Add a new marketplace source
    Add {
        #[arg(value_name = "URL")]
        url: String,
        #[arg(long)]
        display_name: Option<String>,
        #[arg(long)]
        requires_auth: bool,
    },
    /// List configured marketplace sources
    List {
        #[arg(long)]
        json: bool,
    },
    /// Remove a marketplace source
    Remove {
        #[arg(value_name = "SLUG")]
        slug: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = MarketplaceCli::parse();
    let result = match cli.command {
        MarketplaceCommand::List {
            search,
            category,
            source,
            installed,
            json,
        } => {
            execute_list(ListOptions {
                search,
                category,
                source,
                installed_only: installed,
                json,
            })
            .await
        }
        MarketplaceCommand::Show { .. } => Ok(()),
        MarketplaceCommand::Sources { .. } => Ok(()),
        MarketplaceCommand::Refresh { .. } => Ok(()),
        MarketplaceCommand::Status { .. } => Ok(()),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}
