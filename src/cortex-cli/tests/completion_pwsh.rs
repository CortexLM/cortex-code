use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn completion_pwsh_is_accepted_as_powershell_alias() {
    let home = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_Cortex"))
        .args(["completion", "pwsh", "--install"])
        .env("HOME", home.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "completion pwsh failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let profile = home
        .path()
        .join(".config/powershell/Microsoft.PowerShell_profile.ps1");
    let profile = fs::read_to_string(profile).unwrap();
    assert!(
        profile.contains("cortex completion powershell"),
        "profile:\n{profile}"
    );
}
