use std::env;
use std::sync::{Mutex, OnceLock};

use assert_cmd::Command;
use tempfile::TempDir;

const HOME_ENV: &str = "GEMINI_MARKETPLACE_HOME";

/// Global lock to prevent concurrent mutation of GEMINI_MARKETPLACE_HOME.
pub fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Test harness that configures an isolated marketplace home directory.
pub struct MarketplaceTestContext {
    _home: TempDir,
}

impl MarketplaceTestContext {
    /// Create a new temporary home directory and set the override environment variable.
    pub fn new() -> Self {
        let home = TempDir::new().expect("temp dir");
        let path = home
            .path()
            .to_str()
            .expect("temp path utf8 for env override")
            .to_string();
        env::set_var(HOME_ENV, &path);
        Self { _home: home }
    }

    /// Obtain a configured command handle for invoking the marketplace binary.
    pub fn command(&self) -> Command {
        let mut cmd = Command::cargo_bin("gemini-marketplace").expect("binary exists");
        cmd.env(HOME_ENV, env::var(HOME_ENV).expect("home env set"));
        cmd
    }
}

impl Drop for MarketplaceTestContext {
    fn drop(&mut self) {
        env::remove_var(HOME_ENV);
    }
}

#[test]
fn extension_binary_runs() {
    let _guard = env_lock().lock().unwrap();
    let ctx = MarketplaceTestContext::new();
    ctx.command().assert().success();
}
