//! The preferences service, which handles user-configurable settings like the cache TTL.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::marketplace::cache::init;
use crate::marketplace::config::Config;
use crate::marketplace::models::domain::{OutputFormat, SearchMode, UserPreferences};

const PREFERENCES_FILE: &str = "preferences.json";

#[derive(Clone)]
pub struct PreferencesService {
    prefs: UserPreferences,
    path: Option<PathBuf>,
}

impl PreferencesService {
    /// Constructs a preferences service from explicit values.
    ///
    /// This is used mainly in tests.
    pub fn new(prefs: UserPreferences) -> Self {
        Self { prefs, path: None }
    }

    /// Loads preferences from the configuration directory, creating a persisted default if missing.
    pub fn load(config: &Config) -> Result<Self> {
        init::ensure_layout(config)?;
        let path = config.config_dir().join(PREFERENCES_FILE);
        let prefs = if path.exists() {
            read_preferences(&path)?
        } else {
            let defaults = UserPreferences::default();
            write_preferences(&path, &defaults)?;
            defaults
        };
        Ok(Self {
            prefs,
            path: Some(path),
        })
    }

    /// Persists the current preferences to disk.
    ///
    /// This is a no-op if the service was created without a path.
    pub fn save(&self) -> Result<()> {
        if let Some(path) = &self.path {
            write_preferences(path, &self.prefs)?;
        }
        Ok(())
    }

    pub fn cache_ttl_hours(&self) -> u16 {
        self.prefs.cache_ttl_hours
    }

    pub fn search_mode(&self) -> SearchMode {
        self.prefs.search_mode.clone()
    }

    pub fn output_format(&self) -> OutputFormat {
        self.prefs.output_format.clone()
    }

    pub fn auto_refresh_on_launch(&self) -> bool {
        self.prefs.auto_refresh_on_launch
    }

    pub fn preferences(&self) -> &UserPreferences {
        &self.prefs
    }

    pub fn set_cache_ttl_hours(&mut self, hours: u16) {
        self.prefs.cache_ttl_hours = hours;
    }

    pub fn set_search_mode(&mut self, mode: SearchMode) {
        self.prefs.search_mode = mode;
    }

    pub fn set_output_format(&mut self, format: OutputFormat) {
        self.prefs.output_format = format;
    }

    pub fn set_auto_refresh_on_launch(&mut self, enabled: bool) {
        self.prefs.auto_refresh_on_launch = enabled;
    }

    pub fn update(&mut self, prefs: UserPreferences) {
        self.prefs = prefs;
    }

    /// Updates the preferences and persists the change if the service is backed by disk.
    pub fn update_and_save(&mut self, prefs: UserPreferences) -> Result<()> {
        self.update(prefs);
        self.save()
    }
}

fn read_preferences(path: &Path) -> Result<UserPreferences> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read preferences file {}", path.display()))?;
    let prefs = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse preferences file {}", path.display()))?;
    Ok(prefs)
}

fn write_preferences(path: &Path, prefs: &UserPreferences) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create preferences directory {}",
                parent.display()
            )
        })?;
    }
    let contents =
        serde_json::to_string_pretty(prefs).context("Failed to serialize preferences to JSON")?;
    fs::write(path, contents)
        .with_context(|| format!("Failed to write preferences file {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use tempfile::TempDir;

    use super::*;

    use crate::marketplace::services::sources::tests::env_lock;

    fn with_temp_home() -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        let path = temp
            .path()
            .to_str()
            .expect("temp path utf8 for env override")
            .to_string();
        env::set_var("GEMINI_MARKETPLACE_HOME", &path);
        temp
    }

    fn clear_temp_home() {
        env::remove_var("GEMINI_MARKETPLACE_HOME");
    }

    #[test]
    fn load_creates_default_preferences_file() {
        let _guard = env_lock().lock().unwrap();
        let temp = with_temp_home();
        let config = Config::new().expect("config");

        let service = PreferencesService::load(&config).expect("load preferences");
        assert_eq!(service.cache_ttl_hours(), 24);
        assert!(!service.auto_refresh_on_launch());
        assert!(matches!(service.search_mode(), SearchMode::LocalFilter));
        assert!(matches!(service.output_format(), OutputFormat::Table));

        let path = config.config_dir().join(PREFERENCES_FILE);
        assert!(
            path.exists(),
            "expected default preferences file to be created"
        );

        drop(temp);
        clear_temp_home();
    }

    #[test]
    fn save_persists_updated_preferences() {
        let _guard = env_lock().lock().unwrap();
        let temp = with_temp_home();
        let config = Config::new().expect("config");
        let mut service = PreferencesService::load(&config).expect("load preferences");

        service.set_cache_ttl_hours(12);
        service.set_auto_refresh_on_launch(true);
        service.set_search_mode(SearchMode::PreFetchFilter);
        service.set_output_format(OutputFormat::Json);
        service.save().expect("save preferences");

        let reloaded = PreferencesService::load(&config).expect("reload preferences");
        assert_eq!(reloaded.cache_ttl_hours(), 12);
        assert!(reloaded.auto_refresh_on_launch());
        assert!(matches!(reloaded.search_mode(), SearchMode::PreFetchFilter));
        assert!(matches!(reloaded.output_format(), OutputFormat::Json));

        drop(temp);
        clear_temp_home();
    }
}
