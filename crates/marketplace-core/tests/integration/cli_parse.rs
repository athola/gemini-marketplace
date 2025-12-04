use assert_cmd::Command;

#[test]
fn list_command_accepts_search_flags() {
    let mut cmd = Command::cargo_bin("gemini-marketplace").expect("binary exists");
    cmd.args(["list", "--search", "analytics", "--json"])
        .assert()
        .success();
}

#[test]
fn sources_subcommand_structure_parses() {
    let mut cmd = Command::cargo_bin("gemini-marketplace").expect("binary exists");
    cmd.args(["sources", "list", "--json"]).assert().success();
}
