//! Manifest schema + semantic validation pipeline.

use std::time::SystemTime;

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::marketplace::models::domain::{ValidationError, ValidationStatus, ValidationSummary};
use crate::marketplace::models::manifest::{ExtensionManifest, ManifestValidation};

const MAX_NAME_LEN: usize = 80;
const MAX_DESCRIPTION_LEN: usize = 512;
const MAX_CATEGORY_LEN: usize = 32;

/// Return the JSON schema describing `gemini-extension.json` manifests.
pub fn manifest_schema() -> Value {
    serde_json::to_value(schema_for!(ManifestDocumentSchema)).expect("manifest schema serializes")
}

/// Build a validation summary by combining schema warnings and semantic checks.
pub fn build_validation_summary(
    manifest: &ExtensionManifest,
    schema_validation: &ManifestValidation,
) -> ValidationSummary {
    let mut issues = Vec::new();
    for warning in &schema_validation.warnings {
        issues.push(ValidationError::new(
            format!("schema::{}", warning.field),
            &warning.message,
            format!("/{}", warning.field),
        ));
    }

    let semantic_errors = semantic_issues(manifest);
    let semantic_error_count = semantic_errors.len();
    issues.extend(semantic_errors);

    let schema_status = if schema_validation.is_clean() {
        ValidationStatus::Passed
    } else {
        ValidationStatus::Warning
    };

    let semantic_status = if semantic_error_count > 0 {
        ValidationStatus::Failed
    } else {
        ValidationStatus::Passed
    };

    ValidationSummary::new(schema_status, semantic_status, issues, SystemTime::now())
}

fn semantic_issues(manifest: &ExtensionManifest) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let trimmed_name = manifest.name.trim();
    if trimmed_name.is_empty() || trimmed_name.chars().count() > MAX_NAME_LEN {
        errors.push(ValidationError::new(
            "semantic::name",
            format!(
                "Name must be 1-{} characters (got {})",
                MAX_NAME_LEN,
                manifest.name.chars().count()
            ),
            "/name",
        ));
    }

    let description_len = manifest.description.chars().count();
    if description_len == 0 || description_len > MAX_DESCRIPTION_LEN {
        errors.push(ValidationError::new(
            "semantic::description",
            format!(
                "Description must be 1-{} characters (got {})",
                MAX_DESCRIPTION_LEN, description_len
            ),
            "/description",
        ));
    }

    if manifest
        .author
        .as_deref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        errors.push(ValidationError::new(
            "semantic::author",
            "Author is required for provenance.",
            "/author",
        ));
    }

    if manifest.categories.is_empty() {
        errors.push(ValidationError::new(
            "semantic::categories",
            "At least one category is required for discovery.",
            "/categories",
        ));
    }

    for (idx, category) in manifest.categories.iter().enumerate() {
        if category.len() > MAX_CATEGORY_LEN {
            errors.push(ValidationError::new(
                "semantic::category_length",
                format!(
                    "Category '{}' exceeds maximum length of {} characters",
                    category, MAX_CATEGORY_LEN
                ),
                format!("/categories/{idx}"),
            ));
        }
    }

    if manifest.repository.scheme() != "https" {
        errors.push(ValidationError::new(
            "semantic::repository_scheme",
            "Repository URLs must use HTTPS.",
            "/repository",
        ));
    }

    errors
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ManifestDocumentSchema {
    #[schemars(length(min = 1, max = 80))]
    pub name: String,
    #[schemars(length(min = 1, max = 512))]
    pub description: String,
    pub repository: String,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    #[schemars(length(min = 1))]
    pub version: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub compatibility: Vec<String>,
    pub readme: Option<String>,
}
