use std::io;
use std::path::PathBuf;

use gemini_marketplace::marketplace::error::MarketplaceError;

#[test]
fn io_helper_wraps_path_and_source() {
    let err = MarketplaceError::io(PathBuf::from("/tmp/test"), io::Error::from(io::ErrorKind::NotFound));
    match err {
        MarketplaceError::Io { path, source } => {
            assert_eq!(path, PathBuf::from("/tmp/test"));
            assert_eq!(source.kind(), io::ErrorKind::NotFound);
        }
        other => panic!("expected Io variant, got {:?}", other),
    }
}

#[test]
fn configuration_helper_creates_configuration_variant() {
    let err = MarketplaceError::configuration("missing value");
    match err {
        MarketplaceError::Configuration(msg) => assert_eq!(msg, "missing value"),
        other => panic!("expected Configuration variant, got {:?}", other),
    }
}
