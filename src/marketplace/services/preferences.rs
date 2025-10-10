//! Preferences service handles user-configurable settings like cache TTL.

use crate::marketplace::models::domain::{OutputFormat, SearchMode, UserPreferences};

#[derive(Clone)]
pub struct PreferencesService {
    prefs: UserPreferences,
}

impl PreferencesService {
    pub fn new(prefs: UserPreferences) -> Self {
        Self { prefs }
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

    pub fn update(&mut self, prefs: UserPreferences) {
        self.prefs = prefs;
    }
}
