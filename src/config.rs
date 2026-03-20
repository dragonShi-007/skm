use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::platform;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// "project" or a model name (e.g. "cc", "cursor")
    #[serde(default = "default_target")]
    pub default_target: String,
    /// User-defined model name → skills directory path
    #[serde(default)]
    pub models: HashMap<String, String>,
}

fn default_target() -> String {
    "cc".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_target: "cc".to_string(),
            models: HashMap::new(),
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".config").join("skm").join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
    Ok(config)
}

/// Called once at startup. If the config file does not yet exist, write a
/// commented default so the user can see all available options.
pub fn ensure_initialized() -> Result<()> {
    let path = config_path()?;
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = r#"# skm — Skill Manager configuration
#
# default_target: which location is pre-selected in the interactive menu.
#   Use a model name (e.g. "cc") or "project" (.claude/skills/ under cwd).
default_target = "cc"

# Model definitions: a short name mapped to a skills directory path.
# The built-in "cc" model (~/.claude/skills/) is always available.
# Add more with: skm model add <name> <path>
[models]
"#;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;
    println!("Config initialized: {}", path.display());
    Ok(())
}

pub fn save(cfg: &Config) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string(cfg).context("Failed to serialize config")?;
    std::fs::write(&path, contents)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;
    Ok(())
}

fn expand_tilde(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(s)
}

/// Resolve a model name to its skills directory.
/// Built-in: "cc" → ~/.claude/skills/
/// User-defined: looked up from config.models
pub fn resolve_model(name: &str) -> Result<PathBuf> {
    if name == "cc" {
        return platform::cc_skills_dir();
    }
    let cfg = load()?;
    if let Some(path_str) = cfg.models.get(name) {
        return Ok(expand_tilde(path_str));
    }
    anyhow::bail!(
        "Unknown model '{}'. Run 'skm model list' to see available models, \
         or 'skm model add {} <path>' to define one.",
        name, name
    )
}

/// Returns all models in display order: built-ins first, then user-defined sorted by name.
pub fn all_models() -> Result<Vec<(String, PathBuf)>> {
    let mut result = vec![("cc".to_string(), platform::cc_skills_dir()?)];
    let cfg = load()?;
    let mut user: Vec<(String, PathBuf)> = cfg
        .models
        .into_iter()
        .map(|(k, v)| (k, expand_tilde(&v)))
        .collect();
    user.sort_by(|a, b| a.0.cmp(&b.0));
    result.extend(user);
    Ok(result)
}
