use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

pub const DEFAULT_PROFILE_ID: &str = "default";
const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopConfig {
    pub enabled: bool,
    /// Absolute, canonical path passed as --user-data-dir.
    pub user_data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeConfig {
    pub enabled: bool,
    /// Absolute, canonical path exported as CLAUDE_CONFIG_DIR. The macOS
    /// keychain entry name is derived from this exact string — never
    /// rewrite it once credentials exist.
    pub config_dir: PathBuf,
    pub shim_name: String,
    pub shim_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub color: String,
    pub is_default: bool,
    pub desktop: DesktopConfig,
    pub code: CodeConfig,
    pub created_at: String,
    pub last_launched_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub claude_desktop_path: Option<PathBuf>,
    pub claude_code_path: Option<PathBuf>,
    pub shim_dir: Option<PathBuf>,
    pub terminal: String,
    pub hide_window_on_close: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            claude_desktop_path: None,
            claude_code_path: None,
            shim_dir: None,
            terminal: "auto".into(),
            hide_window_on_close: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub version: u32,
    pub settings: Settings,
    pub profiles: Vec<Profile>,
}

/// Input for creating a profile. Paths are derived from the slug unless
/// explicitly overridden.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProfile {
    pub name: String,
    pub color: Option<String>,
    pub desktop_enabled: bool,
    pub code_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdate {
    pub name: Option<String>,
    pub color: Option<String>,
    pub desktop_enabled: Option<bool>,
    pub code_enabled: Option<bool>,
}

pub struct Store {
    config_path: PathBuf,
    pub config: Config,
}

/// Platform-default locations, injectable for tests.
pub struct Env {
    pub home: PathBuf,
    pub app_support: PathBuf,
    pub default_claude_desktop_data: PathBuf,
    pub default_claude_code_config: PathBuf,
}

impl Env {
    pub fn detect() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::not_found("could not determine home directory"))?;
        // CLAUDE_CONFIG_DIR in the user's own environment overrides the
        // default profile's Claude Code dir.
        let default_code = std::env::var_os("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".claude"));
        #[cfg(target_os = "macos")]
        let app_support = home.join("Library/Application Support");
        #[cfg(target_os = "windows")]
        let app_support = dirs::config_dir()
            .ok_or_else(|| AppError::not_found("could not determine %APPDATA%"))?;
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        let app_support = dirs::config_dir()
            .ok_or_else(|| AppError::not_found("could not determine XDG config dir"))?;

        let default_desktop = app_support.join("Claude");
        Ok(Self {
            home,
            app_support,
            default_claude_desktop_data: default_desktop,
            default_claude_code_config: default_code,
        })
    }

    fn desktop_dir_for(&self, slug: &str) -> PathBuf {
        self.app_support.join(format!("Claude-{slug}"))
    }

    fn code_dir_for(&self, slug: &str) -> PathBuf {
        self.home.join(format!(".claude-{slug}"))
    }
}

pub fn slugify(name: &str) -> String {
    let mut slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').to_string()
}

fn now_iso() -> String {
    // RFC3339 without pulling in chrono: seconds precision is plenty.
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    // Days since epoch → civil date (Howard Hinnant's algorithm).
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

impl Store {
    /// Loads the config from disk, creating a fresh one (with the default
    /// profile) on first run. Unknown future versions are rejected rather
    /// than silently rewritten.
    pub fn load(config_path: PathBuf, env: &Env) -> Result<Self> {
        let config = match std::fs::read_to_string(&config_path) {
            Ok(raw) => {
                let config: Config = serde_json::from_str(&raw)?;
                if config.version > CONFIG_VERSION {
                    return Err(AppError::new(
                        crate::error::ErrorKind::Internal,
                        format!(
                            "config version {} is newer than this app supports ({})",
                            config.version, CONFIG_VERSION
                        ),
                    ));
                }
                config
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Config {
                version: CONFIG_VERSION,
                settings: Settings::default(),
                profiles: vec![Self::default_profile(env)],
            },
            Err(e) => return Err(e.into()),
        };
        let mut store = Self {
            config_path,
            config,
        };
        store.ensure_default_profile(env);
        Ok(store)
    }

    fn default_profile(env: &Env) -> Profile {
        Profile {
            id: DEFAULT_PROFILE_ID.into(),
            name: "Default".into(),
            slug: DEFAULT_PROFILE_ID.into(),
            color: "#D97757".into(),
            is_default: true,
            desktop: DesktopConfig {
                enabled: true,
                user_data_dir: env.default_claude_desktop_data.clone(),
            },
            code: CodeConfig {
                enabled: true,
                config_dir: env.default_claude_code_config.clone(),
                shim_name: String::new(),
                shim_installed: false,
            },
            created_at: now_iso(),
            last_launched_at: None,
        }
    }

    fn ensure_default_profile(&mut self, env: &Env) {
        if !self.config.profiles.iter().any(|p| p.is_default) {
            self.config.profiles.insert(0, Self::default_profile(env));
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(&self.config)?;
        // Write-then-rename so a crash can't truncate the config.
        let tmp = self.config_path.with_extension("json.tmp");
        std::fs::write(&tmp, raw)?;
        std::fs::rename(&tmp, &self.config_path)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<&Profile> {
        self.config
            .profiles
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| AppError::not_found(format!("profile {id} not found")))
    }

    fn get_mut(&mut self, id: &str) -> Result<&mut Profile> {
        self.config
            .profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| AppError::not_found(format!("profile {id} not found")))
    }

    pub fn create(&mut self, input: NewProfile, env: &Env) -> Result<Profile> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(AppError::validation("profile name must not be empty"));
        }
        let slug = slugify(name);
        if slug.is_empty() {
            return Err(AppError::validation(
                "profile name must contain at least one letter or digit",
            ));
        }
        if slug == DEFAULT_PROFILE_ID {
            return Err(AppError::validation("'default' is reserved"));
        }
        if self.config.profiles.iter().any(|p| p.slug == slug) {
            return Err(AppError::validation(format!(
                "a profile with slug '{slug}' already exists"
            )));
        }
        let profile = Profile {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            slug: slug.clone(),
            color: input.color.unwrap_or_else(|| "#6A9BCC".into()),
            is_default: false,
            desktop: DesktopConfig {
                enabled: input.desktop_enabled,
                user_data_dir: env.desktop_dir_for(&slug),
            },
            code: CodeConfig {
                enabled: input.code_enabled,
                config_dir: env.code_dir_for(&slug),
                shim_name: format!("claude-{slug}"),
                shim_installed: false,
            },
            created_at: now_iso(),
            last_launched_at: None,
        };
        self.config.profiles.push(profile.clone());
        self.save()?;
        Ok(profile)
    }

    pub fn update(&mut self, id: &str, patch: ProfileUpdate) -> Result<Profile> {
        {
            let is_default = self.get(id)?.is_default;
            let p = self.get_mut(id)?;
            if let Some(name) = patch.name {
                let name = name.trim();
                if name.is_empty() {
                    return Err(AppError::validation("profile name must not be empty"));
                }
                // Renaming changes the display name only; slug and dirs are
                // immutable because keychain entries derive from the paths.
                p.name = name.to_string();
            }
            if let Some(color) = patch.color {
                p.color = color;
            }
            if !is_default {
                if let Some(enabled) = patch.desktop_enabled {
                    p.desktop.enabled = enabled;
                }
                if let Some(enabled) = patch.code_enabled {
                    p.code.enabled = enabled;
                }
            }
        }
        self.save()?;
        Ok(self.get(id)?.clone())
    }

    pub fn remove(&mut self, id: &str) -> Result<Profile> {
        let profile = self.get(id)?.clone();
        if profile.is_default {
            return Err(AppError::validation(
                "the default profile cannot be deleted",
            ));
        }
        self.config.profiles.retain(|p| p.id != id);
        self.save()?;
        Ok(profile)
    }

    pub fn touch_launched(&mut self, id: &str) -> Result<()> {
        self.get_mut(id)?.last_launched_at = Some(now_iso());
        self.save()
    }
}

/// First 8 hex chars of sha256 over the exact config-dir path string —
/// the suffix Claude Code appends to its macOS Keychain service name.
pub fn keychain_suffix(config_dir: &Path) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(config_dir.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    digest[..4].iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_env(root: &Path) -> Env {
        Env {
            home: root.join("home"),
            app_support: root.join("appsupport"),
            default_claude_desktop_data: root.join("appsupport/Claude"),
            default_claude_code_config: root.join("home/.claude"),
        }
    }

    fn temp_store() -> (tempfile::TempDir, Store, Env) {
        let dir = tempfile::tempdir().unwrap();
        let env = test_env(dir.path());
        let store = Store::load(dir.path().join("config.json"), &env).unwrap();
        (dir, store, env)
    }

    #[test]
    fn fresh_store_has_default_profile() {
        let (_dir, store, _env) = temp_store();
        assert_eq!(store.config.profiles.len(), 1);
        let p = &store.config.profiles[0];
        assert!(p.is_default);
        assert_eq!(p.id, DEFAULT_PROFILE_ID);
    }

    #[test]
    fn create_and_reload_round_trip() {
        let (dir, mut store, env) = temp_store();
        store
            .create(
                NewProfile {
                    name: "Work Stuff".into(),
                    color: None,
                    desktop_enabled: true,
                    code_enabled: true,
                },
                &env,
            )
            .unwrap();
        let reloaded = Store::load(dir.path().join("config.json"), &env).unwrap();
        assert_eq!(reloaded.config.profiles.len(), 2);
        let p = &reloaded.config.profiles[1];
        assert_eq!(p.slug, "work-stuff");
        assert_eq!(p.code.shim_name, "claude-work-stuff");
        assert!(p.desktop.user_data_dir.ends_with("Claude-work-stuff"));
        assert!(p.code.config_dir.ends_with(".claude-work-stuff"));
    }

    #[test]
    fn slug_collision_rejected() {
        let (_dir, mut store, env) = temp_store();
        let input = NewProfile {
            name: "Work".into(),
            color: None,
            desktop_enabled: true,
            code_enabled: false,
        };
        store.create(input.clone(), &env).unwrap();
        let err = store.create(input, &env).unwrap_err();
        assert!(err.message.contains("already exists"));
    }

    #[test]
    fn default_profile_cannot_be_deleted() {
        let (_dir, mut store, _env) = temp_store();
        assert!(store.remove(DEFAULT_PROFILE_ID).is_err());
    }

    #[test]
    fn rename_keeps_slug_and_paths() {
        let (_dir, mut store, env) = temp_store();
        let p = store
            .create(
                NewProfile {
                    name: "Work".into(),
                    color: None,
                    desktop_enabled: true,
                    code_enabled: true,
                },
                &env,
            )
            .unwrap();
        let before_dir = p.code.config_dir.clone();
        let updated = store
            .update(
                &p.id,
                ProfileUpdate {
                    name: Some("Job".into()),
                    color: None,
                    desktop_enabled: None,
                    code_enabled: None,
                },
            )
            .unwrap();
        assert_eq!(updated.name, "Job");
        assert_eq!(updated.slug, "work");
        assert_eq!(updated.code.config_dir, before_dir);
    }

    #[test]
    fn slugify_cases() {
        assert_eq!(slugify("Work"), "work");
        assert_eq!(slugify("Côté & Co."), "c-t-co");
        assert_eq!(slugify("  A  B  "), "a-b");
        assert_eq!(slugify("!!!"), "");
    }

    #[test]
    fn keychain_suffix_matches_known_vector() {
        // Verified live against Claude Code 2.1.220 on macOS.
        assert_eq!(
            keychain_suffix(Path::new("/Users/dominic/.claude-personal")),
            "435d6ab5"
        );
    }

    #[test]
    fn newer_config_version_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let env = test_env(dir.path());
        let path = dir.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"version": 999, "settings": {"claudeDesktopPath": null, "claudeCodePath": null, "shimDir": null, "terminal": "auto", "hideWindowOnClose": true}, "profiles": []}"#,
        )
        .unwrap();
        assert!(Store::load(path, &env).is_err());
    }
}
