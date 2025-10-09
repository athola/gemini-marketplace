use assert_cmd::Command;

#[test]
fn extension_binary_runs() {
    let mut cmd = Command::cargo_bin("gemini-marketplace").expect("binary exists");
    cmd.assert().success();
}
