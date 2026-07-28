use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AppError, Result};

const BUNDLE_ID: &str = "com.anthropic.claudefordesktop";

/// Locate Claude.app: well-known paths first, then Spotlight by bundle id.
pub fn find_claude_desktop() -> Option<PathBuf> {
    for candidate in ["/Applications/Claude.app", "~/Applications/Claude.app"] {
        let path = expand_home(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    let out = Command::new("mdfind")
        .arg(format!("kMDItemCFBundleIdentifier == '{BUNDLE_ID}'"))
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(PathBuf::from)
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

/// Launch Claude Desktop for a profile. The default profile activates the
/// normally-launched instance (no -n) so we never spawn a duplicate of it.
pub fn launch_desktop(app_path: &Path, user_data_dir: &Path, is_default: bool) -> Result<()> {
    let mut cmd = Command::new("open");
    if is_default {
        cmd.arg("-a").arg(app_path);
    } else {
        cmd.arg("-n")
            .arg("-a")
            .arg(app_path)
            .arg("--args")
            .arg(format!("--user-data-dir={}", user_data_dir.display()));
    }
    let status = cmd.status()?;
    if !status.success() {
        return Err(AppError::launch(format!(
            "`open` exited with {status} when launching Claude Desktop"
        )));
    }
    Ok(())
}

/// Find the pid of a running Claude Desktop main process using this exact
/// user-data-dir, if any.
pub fn find_running_desktop(user_data_dir: &Path) -> Option<u32> {
    let needle = format!("--user-data-dir={}", user_data_dir.display());
    let sys = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::nothing().with_processes(
        sysinfo::ProcessRefreshKind::nothing().with_cmd(sysinfo::UpdateKind::Always),
    ));
    sys.processes().values().find_map(|p| {
        let cmd = p.cmd();
        let is_main = cmd
            .first()
            .map(|c| c.to_string_lossy().ends_with("MacOS/Claude"))
            .unwrap_or(false);
        let has_dir = cmd.iter().any(|c| c.to_string_lossy() == needle.as_str());
        // Helper processes also carry the flag; match only the main binary.
        (is_main && has_dir).then(|| p.pid().as_u32())
    })
}

pub fn focus_process(pid: u32) -> Result<()> {
    let script = format!(
        "tell application \"System Events\" to set frontmost of (first process whose unix id is {pid}) to true"
    );
    let status = Command::new("osascript").arg("-e").arg(script).status()?;
    if !status.success() {
        return Err(AppError::launch("could not focus running Claude instance"));
    }
    Ok(())
}

/// Open a terminal window with the tool's config env var exported and the
/// tool started (e.g. CLAUDE_CONFIG_DIR + claude, CODEX_HOME + codex).
pub fn launch_terminal(
    terminal: &str,
    env_var: &str,
    config_dir: &Path,
    binary: &str,
) -> Result<()> {
    let dir = config_dir.display();
    let shell_cmd = format!("export {env_var}=\"{dir}\" && {binary}");
    let script = match terminal {
        "iTerm2" => format!(
            "tell application \"iTerm2\"\n  create window with default profile command \"/bin/zsh -lc 'export {env_var}=\\\"{dir}\\\" && {binary}'\"\n  activate\nend tell"
        ),
        // "auto" and "Terminal" both use Terminal.app.
        _ => format!(
            "tell application \"Terminal\"\n  do script \"{}\"\n  activate\nend tell",
            shell_cmd.replace('\\', "\\\\").replace('"', "\\\"")
        ),
    };
    let status = Command::new("osascript").arg("-e").arg(script).status()?;
    if !status.success() {
        return Err(AppError::launch(format!(
            "osascript failed to open {terminal}"
        )));
    }
    Ok(())
}

/// Default directory for shims.
pub fn default_shim_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".local/bin"))
}

/// Delete the profile's Claude Code credential from the Keychain.
/// A missing entry is not an error.
pub fn delete_keychain_credential(config_dir: &Path) -> Result<bool> {
    let service = format!(
        "Claude Code-credentials-{}",
        crate::profiles::keychain_suffix(config_dir)
    );
    let status = Command::new("security")
        .args(["delete-generic-password", "-s", &service])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    Ok(status.success())
}
