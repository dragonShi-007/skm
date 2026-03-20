use anyhow::Result;
use dialoguer::{Input, Select};
use std::env;
use std::path::PathBuf;

use crate::config;

pub struct ScopeFlags {
    pub model: Option<String>,
    pub project: Option<PathBuf>,
}

pub fn resolve_target(flags: ScopeFlags) -> Result<PathBuf> {
    // Priority 1: -m/--model <name>
    if let Some(name) = flags.model {
        return config::resolve_model(&name);
    }

    // Priority 2: -p [PATH]
    //   -p alone → current directory
    //   -p <path> → use that path directly
    if let Some(p) = flags.project {
        if p == std::path::Path::new(".") {
            return Ok(env::current_dir()?);
        }
        return Ok(p);
    }

    // Priority 3: interactive prompt
    let cfg = config::load()?;
    let models = config::all_models()?;
    let cwd = env::current_dir()?;

    let mut options: Vec<String> = models
        .iter()
        .map(|(name, path)| format!("{} ({})", name, path.display()))
        .collect();
    let model_count = options.len();
    options.push(format!("Project ({})", cwd.display()));
    options.push("Custom path".to_string());

    let default_idx = if cfg.default_target == "project" {
        model_count
    } else {
        models
            .iter()
            .position(|(n, _)| n == &cfg.default_target)
            .unwrap_or(0)
    };

    let selection = Select::new()
        .with_prompt("Select target location")
        .items(&options)
        .default(default_idx)
        .interact()?;

    if selection < model_count {
        Ok(models[selection].1.clone())
    } else if selection == model_count {
        Ok(cwd)
    } else {
        let input: String = Input::new()
            .with_prompt("Enter target directory")
            .interact_text()?;
        Ok(PathBuf::from(input))
    }
}
