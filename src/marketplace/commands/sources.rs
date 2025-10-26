//! Basic CLI handlers for the `sources` command family.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json;
use url::Url;

use crate::marketplace::models::domain::{MarketplaceSource, SourceType, SyncStatus};
use crate::marketplace::services::sources::SourcesService;

#[derive(Serialize)]
struct SourceView<'a> {
    slug: &'a str,
    display_name: &'a str,
    url: &'a str,
    enabled: bool,
    requires_auth: bool,
    source_type: &'a str,
}

pub fn list(json: bool) -> Result<()> {
    let service = SourcesService::new()?;
    let sources = service.list_sources()?;

    if json {
        let payload: Vec<SourceView<'_>> = sources
            .iter()
            .map(|s| SourceView {
                slug: &s.slug,
                display_name: &s.display_name,
                url: s.url.as_str(),
                enabled: s.enabled,
                requires_auth: s.requires_auth,
                source_type: match s.source_type {
                    SourceType::GithubRepo => "github_repo",
                    SourceType::GitUrl => "git_url",
                    SourceType::LocalPath => "local_path",
                },
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if sources.is_empty() {
        println!("No marketplace sources configured.");
        return Ok(());
    }

    println!("Marketplace Sources:\n");
    for source in sources {
        println!(
            "- {slug} ({display})\n  URL: {url}\n  Enabled: {enabled}\n  Requires Auth: {auth}\n",
            slug = source.slug,
            display = source.display_name,
            url = source.url,
            enabled = if source.enabled { "yes" } else { "no" },
            auth = if source.requires_auth { "yes" } else { "no" }
        );
    }

    Ok(())
}

pub fn add(url: &str, display_name: Option<String>, requires_auth: bool) -> Result<()> {
    let service = SourcesService::new()?;

    let parsed = Url::parse(url).context("invalid source URL")?;
    let slug = derive_slug(&parsed)?;
    let name = display_name.unwrap_or_else(|| slug.to_string());
    let source_type = infer_source_type(&parsed);

    let source = MarketplaceSource::new(slug, name, parsed, source_type, true, 5)
        .with_sync_status(SyncStatus::Idle)
        .with_requires_auth(requires_auth);

    service.add_source(source)?;
    println!("Source added successfully.");
    Ok(())
}

pub fn remove(slug: &str) -> Result<()> {
    let service = SourcesService::new()?;
    service.remove_source(slug)?;
    println!("Removed source '{slug}'.");
    Ok(())
}

fn derive_slug(url: &Url) -> Result<String> {
    if let Some(last) = url
        .path_segments()
        .and_then(|segments| segments.filter(|s| !s.is_empty()).last())
    {
        return Ok(last.trim_end_matches(".git").to_lowercase());
    }
    if let Some(host) = url.host_str() {
        return Ok(host.to_lowercase().replace('.', "-"));
    }
    Err(anyhow!("unable to derive slug from URL"))
}

fn infer_source_type(url: &Url) -> SourceType {
    match url.scheme() {
        "http" | "https" => SourceType::GithubRepo,
        _ => SourceType::GitUrl,
    }
}
