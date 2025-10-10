use gemini_marketplace::marketplace::models::manifest::ExtensionManifest;

#[test]
fn normalizes_collections_and_records_warnings() {
    let json = r#"{
        "name": "demo",
        "description": "Demo extension",
        "repository": "https://github.com/example/demo",
        "version": "1.2.3",
        "author": "Example",
        "categories": ["CLI Tools"],
        "tags": ["Utility ", " CLI"],
        "compatibility": ["cli>=1.0"]
    }"#;

    let (manifest, validation) = ExtensionManifest::from_str(json, "example/demo").expect("manifest parsed");

    assert_eq!(manifest.categories, vec!["cli-tools"]);
    assert_eq!(manifest.tags, vec!["utility", "cli"]);
    assert!(validation
        .warnings
        .iter()
        .any(|w| w.field == "categories" && w.message.contains("kebab-case")));
}

#[test]
fn missing_required_fields_emit_warnings() {
    let json = r#"{
        "name": "",
        "description": "",
        "repository": "https://github.com/example/demo",
        "version": "1.2.3"
    }"#;

    let (_manifest, validation) = ExtensionManifest::from_str(json, "example/demo").expect("manifest parsed");

    let fields: Vec<_> = validation.warnings.iter().map(|w| w.field.as_str()).collect();
    assert!(fields.contains(&"name"));
    assert!(fields.contains(&"description"));
    assert!(fields.contains(&"author"));
    assert!(fields.contains(&"categories"));
}

#[test]
fn invalid_json_returns_error() {
    let json = "{ invalid";
    let err = ExtensionManifest::from_str(json, "example/demo").expect_err("should fail");
    let message = format!("{}", err);
    assert!(message.contains("invalid JSON"));
}
