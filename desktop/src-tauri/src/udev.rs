// Bigscreen Beyond udev rule install.
//
// Controlling the Beyond's brightness/fans needs write access to its hidraw
// node. We grant it with a udev rule using `TAG+="uaccess"`, which gives the
// active logged-in user access (no group membership required). Writing to
// /etc/udev/rules.d needs root, so we elevate with `pkexec` (the graphical
// polkit prompt) and reload + re-trigger udev so the running device picks it up.
use std::process::Command;

const RULE_PATH: &str = "/etc/udev/rules.d/70-nemurixr-beyond.rules";
const RULE: &str =
    r#"KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="35bd", ATTRS{idProduct}=="0101", TAG+="uaccess""#;

/// The rule text (also shown in the UI for the manual fallback).
pub fn rule_text() -> String {
    format!("{RULE}\n")
}

/// Install the rule via pkexec, then reload + trigger udev. Blocking (waits on
/// the polkit dialog), so call it off the main thread.
pub fn install() -> Result<(), String> {
    // Stage the rule in a user-writable temp file, then have the privileged
    // shell copy it into place (avoids quoting the rule through pkexec).
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let tmp = format!("{dir}/nemurixr-beyond.rules");
    std::fs::write(&tmp, rule_text()).map_err(|e| format!("Couldn't stage the rule: {e}"))?;

    let script = format!(
        "install -m 0644 \"$1\" '{RULE_PATH}' && udevadm control --reload-rules && udevadm trigger --subsystem-match=hidraw"
    );
    let status = Command::new("pkexec")
        .arg("/bin/sh")
        .arg("-c")
        .arg(&script)
        .arg("nemurixr") // $0 inside the shell
        .arg(&tmp) // $1
        .status();
    let _ = std::fs::remove_file(&tmp);

    match status {
        Ok(s) if s.success() => Ok(()),
        // pkexec: 126 = not authorized / dismissed, 127 = auth could not be obtained.
        Ok(s) if matches!(s.code(), Some(126) | Some(127)) => Err("Authorization was dismissed or denied".to_string()),
        Ok(_) => Err("Failed to install the rule".to_string()),
        Err(e) => Err(format!("Couldn't run pkexec (is polkit installed?): {e}")),
    }
}
