use gemini_marketplace::marketplace::ingest::validation::{
    build_validation_summary, manifest_schema,
};
use gemini_marketplace::marketplace::models::domain::ValidationStatus;
use gemini_marketplace::marketplace::models::manifest::ExtensionManifest;

#[test]
fn manifest_schema_includes_description_constraints() {
    let schema = manifest_schema();
    let description = schema["properties"]["description"]
        .as_object()
        .expect("description property");
    let max = description
        .get("maxLength")
        .and_then(|value| value.as_u64())
        .expect("max length");
    assert_eq!(max, 512);
}

#[test]
fn semantic_validation_flags_invalid_description() {
    let json = r#"{
        "name": "demo",
        "description": "A very long description that exceeds the allowed limit.................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................................",
        "repository": "https://github.com/example/demo",
        "version": "1.0.0",
        "categories": ["cli"]
    }"#;
    let (manifest, schema_validation) =
        ExtensionManifest::from_str(json, "example/demo").expect("manifest loads");
    let summary = build_validation_summary(&manifest, &schema_validation);
    assert_eq!(summary.semantic_status, ValidationStatus::Failed);
    assert!(!summary.errors.is_empty());
}
