use std::str;

use anyhow::Result;
use assert_cmd::Command;

#[test]
fn cli_help_lists_marketplace_subcommands() -> Result<()> {
    let output = Command::cargo_bin("gemini-marketplace")?
        .arg("--help")
        .output()?;

    assert!(output.status.success(), "help command should succeed");
    let stdout = str::from_utf8(&output.stdout)?;

    for keyword in ["list", "show", "search", "sources", "cache"] {
        assert!(
            stdout.contains(keyword),
            "`--help` output missing expected subcommand `{}`\n{}",
            keyword,
            stdout
        );
    }

    Ok(())
}
