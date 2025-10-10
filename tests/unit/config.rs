use std::env;
use std::path::Path;

use tempfile::tempdir;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn config_uses_override_environment_variable() -> Result<()> {
    let temp = tempdir()?;
    env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());

    let config = gemini_marketplace::marketplace::config::Config::new()?;

    assert!(config.cache_dir().starts_with(temp.path()));
    assert!(config.config_dir().starts_with(temp.path()));
    assert!(config.log_dir().starts_with(temp.path()));

    // ensure directories are created under the override path
    config.ensure_dirs()?;
    assert!(config.cache_dir().is_dir());
    assert!(config.config_dir().is_dir());
    assert!(config.log_dir().is_dir());

    env::remove_var("GEMINI_MARKETPLACE_HOME");
    Ok(())
}

#[test]
fn config_defaults_to_project_dirs() -> Result<()> {
    env::remove_var("GEMINI_MARKETPLACE_HOME");
    let config = gemini_marketplace::marketplace::config::Config::new()?;

    let cache = config.cache_dir();
    let config_dir = config.config_dir();
    let log_dir = config.log_dir();

    for path in [&cache, &config_dir, &log_dir] {
        assert!(path.is_absolute());
        assert!(Path::new(path).to_path_buf().components().count() > 1);
    }

    Ok(())
}
