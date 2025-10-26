use std::process::ExitCode;

use clap::{Parser, Subcommand};

use gemini_marketplace::marketplace::commands::cache::{
    execute_refresh as execute_cache_refresh, execute_ttl_set as execute_cache_ttl_set,
    CacheRefreshOptions, CacheTtlSetOptions,
};
use gemini_marketplace::marketplace::commands::list::{execute as execute_list, ListOptions};
use gemini_marketplace::marketplace::commands::search::{execute as execute_search, SearchOptions};
use gemini_marketplace::marketplace::commands::sources;
use gemini_marketplace::marketplace::error::MarketplaceError;

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
    /// Search extensions by keyword or category
    Search {
        #[arg(value_name = "KEYWORD")]
        keyword: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        installed: bool,
        #[arg(long)]
        json: bool,
    },
    /// Manage marketplace sources
    Sources {
        #[command(subcommand)]
        subcommand: SourcesCommand,
    },
    /// Manage cache operations
    Cache {
        #[command(subcommand)]
        subcommand: CacheCommand,
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

#[derive(Debug, Subcommand)]
enum CacheCommand {
    /// Refresh marketplace cache
    Refresh {
        #[arg(long)]
        force: bool,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Configure cache TTL (hours)
    Ttl {
        #[command(subcommand)]
        subcommand: CacheTtlCommand,
    },
}

#[derive(Debug, Subcommand)]
enum CacheTtlCommand {
    /// Set cache TTL in hours
    Set {
        #[arg(value_name = "HOURS")]
        hours: u16,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let _ = tracing_subscriber::fmt::try_init();
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
        MarketplaceCommand::Search {
            keyword,
            category,
            source,
            installed,
            json,
        } => {
            execute_search(SearchOptions {
                keyword,
                category,
                source,
                installed_only: installed,
                json,
            })
            .await
        }
        MarketplaceCommand::Show { .. } => Ok(()),
        MarketplaceCommand::Sources { subcommand } => match subcommand {
            SourcesCommand::Add {
                url,
                display_name,
                requires_auth,
            } => sources::add(&url, display_name, requires_auth),
            SourcesCommand::List { json } => sources::list(json),
            SourcesCommand::Remove { slug } => sources::remove(&slug),
        },
        MarketplaceCommand::Cache { subcommand } => match subcommand {
            CacheCommand::Refresh {
                force,
                source,
                json,
            } => {
                execute_cache_refresh(CacheRefreshOptions {
                    force,
                    source,
                    json,
                })
                .await
            }
            CacheCommand::Ttl {
                subcommand: CacheTtlCommand::Set { hours },
            } => execute_cache_ttl_set(CacheTtlSetOptions { hours }).await,
        },
        MarketplaceCommand::Status { .. } => Ok(()),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            match err.downcast::<MarketplaceError>() {
                Ok(market_err) => report_marketplace_error(market_err),
                Err(other) => eprintln!("Error: {other}"),
            }
            ExitCode::FAILURE
        }
    }
}

fn report_marketplace_error(error: MarketplaceError) {
    match error {
        MarketplaceError::RateLimited { source_slug, reset_at } => {
            match reset_at {
                Some(ts) => eprintln!(
                    "Rate limit exceeded for source '{source_slug}'. Try again after {ts}, or rerun with cached data if available."
                ),
                None => eprintln!(
                    "Rate limit exceeded for source '{source_slug}'. Try again in a few minutes."
                ),
            }
        }
        MarketplaceError::Network {
            operation,
            source,
            url,
        } => {
            let message = source.to_string();
            if let Some(url) = url {
                eprintln!(
                    "Network error during {operation} while contacting {url}: {message}"
                );
            } else {
                eprintln!("Network error during {operation}: {message}");
            }
        }
        MarketplaceError::SourceNotFound { slug } => {
            eprintln!(
                "Source '{slug}' not found. Run `gemini-marketplace sources list` to review configured sources."
            );
        }
        MarketplaceError::AuthenticationRequired { slug } => {
            eprintln!(
                "Source '{slug}' requires authentication. Add credentials via `gemini-marketplace sources add --requires-auth` or update your config."
            );
        }
        MarketplaceError::ExtensionNotFound { id } => {
            eprintln!(
                "Extension '{id}' not found. Check the identifier or refresh with `gemini-marketplace cache refresh`."
            );
        }
        MarketplaceError::Configuration(message) => {
            eprintln!("Configuration error: {message}");
        }
        other => eprintln!("Error: {other}"),
    }
}
