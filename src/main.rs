mod cli;
mod config;
mod github;
mod platform;
mod prompt;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, ConfigAction};
use prompt::ScopeFlags;

#[tokio::main]
async fn main() -> Result<()> {
    config::ensure_initialized()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { url, model, project } => {
            let target = prompt::resolve_target(ScopeFlags { model, project })?;
            github::install(&url, &target).await?;
        }
        Commands::List { model, project } => {
            let target = prompt::resolve_target(ScopeFlags { model, project })?;
            platform::list_skills(&target)?;
        }
        Commands::Uninstall { name, model, project } => {
            let target = prompt::resolve_target(ScopeFlags { model, project })?;
            platform::uninstall(&name, &target)?;
        }
        Commands::Update { name, model, project } => {
            let target = prompt::resolve_target(ScopeFlags { model, project })?;
            github::update(name.as_deref(), &target).await?;
        }
        Commands::Config { action } => match action {
            ConfigAction::Show => {
                let cfg = config::load()?;
                let path = config::config_path()?;
                println!("Config file: {}", path.display());
                println!("default_target = {}", cfg.default_target);
                println!("models:");
                println!("  cc (built-in): {}", platform::cc_skills_dir()?.display());
                let mut entries: Vec<_> = cfg.models.iter().collect();
                entries.sort_by_key(|(k, _)| k.as_str());
                for (name, path) in entries {
                    println!("  {}: {}", name, path);
                }
            }
            ConfigAction::Remove { name } => {
                if name == "cc" {
                    anyhow::bail!("Cannot remove the built-in 'cc' model.");
                }
                let mut cfg = config::load()?;
                if cfg.models.remove(&name).is_none() {
                    anyhow::bail!(
                        "Model '{}' not found. Run 'skm config show' to see defined models.",
                        name
                    );
                }
                if cfg.default_target == name {
                    cfg.default_target = "cc".to_string();
                    println!("default_target reset to 'cc'");
                }
                config::save(&cfg)?;
                println!("Removed model '{}'", name);
            }
            ConfigAction::Set { key, value } => match key.as_str() {
                "default-target" => {
                    let mut cfg = config::load()?;
                    cfg.default_target = value.clone();
                    config::save(&cfg)?;
                    println!("Set default_target = {}", value);
                }
                name => {
                    if name == "cc" {
                        anyhow::bail!("Cannot override the built-in 'cc' model.");
                    }
                    let mut cfg = config::load()?;
                    cfg.models.insert(name.to_string(), value.clone());
                    config::save(&cfg)?;
                    println!("Set model '{}' -> {}", name, value);
                }
            },
        },
    }

    Ok(())
}
