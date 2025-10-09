//! Raw manifest representation parsed from `gemini-extension.json`.
//!
//! Provides helpers to load and validate manifests while normalizing fields
//! that must conform to marketplace expectations (e.g., kebab-case categories).
//! Validation collects warnings instead of failing fast so callers can decide
//! whether to skip or continue processing a source.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use semver::Version;
use serde::de::{self, Deserializer};
use serde::Deserialize;
use url::Url;

use crate::marketplace::error::{MarketplaceError, Result};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub description: String,
    #[serde(deserialize_with = "deserialize_url")]
    pub repository: Url,
    #[serde(default, deserialize_with = "deserialize_optional_url")]
    pub homepage: Option<Url>,
    #[serde(default, deserialize_with = "deserialize_optional_url")]
    pub documentation: Option<Url>,
    #[serde(deserialize_with = "deserialize_version")]
    pub version: Version,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub compatibility: Vec<String>,
    #[serde(default)]
    pub readme: Option<String>,
}

#[derive(Debug, Default)]
pub struct ManifestValidation {
    pub warnings: Vec<ManifestWarning>,
}

#[derive(Debug)]
pub struct ManifestWarning {
    pub field: String,
    pub message: String,
}

impl ManifestValidation {
    pub fn push(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.warnings.push(ManifestWarning {
            field: field.into(),
            message: message.into(),
        });
    }

    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty()
    }
}

impl ExtensionManifest {
    /// Load a manifest from disk, returning the parsed manifest and validation warnings.
    pub fn load(path: impl AsRef<Path>) -> Result<(Self, ManifestValidation)> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)
            .map_err(|err| MarketplaceError::io(path_ref.to_path_buf(), err))?;
        let reader = BufReader::new(file);
        let mut manifest: ExtensionManifest =
            serde_json::from_reader(reader).map_err(|err| MarketplaceError::InvalidManifest {
                repository: path_ref.display().to_string(),
                reason: format!("invalid JSON: {err}"),
            })?;
        let mut validation = ManifestValidation::default();
        manifest.normalize(&mut validation);
        manifest.check_required_fields(&mut validation);
        Ok((manifest, validation))
    }

    /// Parse a manifest from JSON string content.
    pub fn from_str(content: &str, source: &str) -> Result<(Self, ManifestValidation)> {
        let mut manifest: ExtensionManifest =
            serde_json::from_str(content).map_err(|err| MarketplaceError::InvalidManifest {
                repository: source.to_string(),
                reason: format!("invalid JSON: {err}"),
            })?;
        let mut validation = ManifestValidation::default();
        manifest.normalize(&mut validation);
        manifest.check_required_fields(&mut validation);
        Ok((manifest, validation))
    }

    fn normalize(&mut self, validation: &mut ManifestValidation) {
        self.categories = normalize_collection("categories", &self.categories, validation);
        self.tags = normalize_collection("tags", &self.tags, validation);
    }

    fn check_required_fields(&self, validation: &mut ManifestValidation) {
        if self.name.trim().is_empty() {
            validation.push("name", "Extension name must be provided");
        }
        if self.description.trim().is_empty() {
            validation.push("description", "Extension description must not be empty");
        }
        if self.author.as_deref().map_or(true, str::is_empty) {
            validation.push(
                "author",
                "Author or maintainer should be specified to aid discovery",
            );
        }
        if self.categories.is_empty() {
            validation.push(
                "categories",
                "At least one category should be provided for filtering",
            );
        }
    }
}

fn normalize_collection(
    field: &'static str,
    values: &[String],
    validation: &mut ManifestValidation,
) -> Vec<String> {
    values
        .iter()
        .map(|value| {
            let trimmed = value.trim();
            let normalized = trimmed.replace(' ', "-").to_lowercase();
            if normalized != trimmed {
                validation.push(
                    field,
                    format!(
                        "Value \"{trimmed}\" normalized to \"{normalized}\"; ensure manifest uses kebab-case"
                    ),
                );
            }
            normalized
        })
        .collect()
}

fn deserialize_url<'de, D>(deserializer: D) -> std::result::Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Url::parse(&s).map_err(|err| de::Error::custom(format!("invalid URL {s}: {err}")))
}

fn deserialize_optional_url<'de, D>(deserializer: D) -> std::result::Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    opt.map_or(Ok(None), |s| {
        if s.trim().is_empty() {
            Ok(None)
        } else {
            Url::parse(&s)
                .map(Some)
                .map_err(|err| de::Error::custom(format!("invalid URL {s}: {err}")))
        }
    })
}

fn deserialize_version<'de, D>(deserializer: D) -> std::result::Result<Version, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Version::parse(&s).map_err(|err| de::Error::custom(format!("invalid SemVer {s}: {err}")))
}
