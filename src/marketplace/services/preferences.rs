//! Preferences service handles user-configurable settings like cache TTL.

use crate::marketplace::models::domain::UserPreferences;

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

    #[allow(dead_code)]
    pub fn update(&mut self, prefs: UserPreferences) {
        self.prefs = prefs;
    }
}
