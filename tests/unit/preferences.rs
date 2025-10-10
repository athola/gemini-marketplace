use gemini_marketplace::marketplace::models::domain::{OutputFormat, SearchMode, UserPreferences};
use gemini_marketplace::marketplace::services::preferences::PreferencesService;

#[test]
fn preferences_service_exposes_fields() {
    let prefs = UserPreferences {
        cache_ttl_hours: 12,
        auto_refresh_on_launch: true,
        search_mode: SearchMode::PreFetchFilter,
        output_format: OutputFormat::Json,
    };
    let service = PreferencesService::new(prefs.clone());

    assert_eq!(service.cache_ttl_hours(), 12);
    assert!(service.auto_refresh_on_launch());
    assert!(matches!(service.search_mode(), SearchMode::PreFetchFilter));
    assert!(matches!(service.output_format(), OutputFormat::Json));

    let mut mutable = service.clone();
    let updated = UserPreferences {
        cache_ttl_hours: 24,
        auto_refresh_on_launch: false,
        search_mode: SearchMode::LocalFilter,
        output_format: OutputFormat::Table,
    };
    mutable.update(updated.clone());
    assert_eq!(mutable.cache_ttl_hours(), 24);
    assert!(!mutable.auto_refresh_on_launch());
    assert!(matches!(mutable.search_mode(), SearchMode::LocalFilter));
    assert!(matches!(mutable.output_format(), OutputFormat::Table));
}
