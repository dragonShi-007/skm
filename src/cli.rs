use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "skm",
    about = "Skill Manager — install and manage AI skills for Claude Code",
    after_help = "Run 'skm <COMMAND> --help' for detailed usage of each command."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a skill from a GitHub URL
    #[command(after_help = "\
TARGET FLAGS (pick at most one):
  -m, --model MODEL  Install to the path registered for MODEL (e.g. cc, cursor)
  -p                 Install to the current directory
  -p <PATH>          Install to the specified path
  (none)             Show an interactive menu; default pre-selected by 'skm config'

EXAMPLES:
  skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming -m cc
  skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming -p
  skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming -p ~/myproject/skills
  skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming -p /tmp/test")]
    Install {
        /// Skill name (e.g. brainstorming) or full GitHub URL
        /// (format: https://github.com/OWNER/REPO/tree/BRANCH/path/to/skill)
        url: String,

        /// Model name whose registered path will be used (e.g. cc, cursor)
        #[arg(short = 'm', long, value_name = "MODEL")]
        model: Option<String>,

        /// Target directory; omit PATH to use current directory
        #[arg(short = 'p', long = "project", num_args = 0..=1, value_name = "PATH",
              default_missing_value = ".")]
        project: Option<PathBuf>,
    },

    /// List installed skills in a target directory
    #[command(after_help = "\
EXAMPLES:
  skm list -m cc
  skm list -p
  skm list -p ~/myproject/skills
  skm list")]
    List {
        /// Model name whose registered path will be used
        #[arg(short = 'm', long, value_name = "MODEL")]
        model: Option<String>,

        /// Target directory; omit PATH to use current directory
        #[arg(short = 'p', long = "project", num_args = 0..=1, value_name = "PATH",
              default_missing_value = ".")]
        project: Option<PathBuf>,
    },

    /// Uninstall a skill by name
    #[command(after_help = "\
EXAMPLES:
  skm uninstall brainstorming -m cc
  skm uninstall brainstorming -p
  skm uninstall brainstorming -p ~/myproject/skills")]
    Uninstall {
        /// Name of the skill to remove (must match the directory name under skills/)
        name: String,

        /// Model name whose registered path will be used
        #[arg(short = 'm', long, value_name = "MODEL")]
        model: Option<String>,

        /// Target directory; omit PATH to use current directory
        #[arg(short = 'p', long = "project", num_args = 0..=1, value_name = "PATH",
              default_missing_value = ".")]
        project: Option<PathBuf>,
    },

    /// Update an installed skill (or all skills) to the latest version
    #[command(after_help = "\
EXAMPLES:
  skm update brainstorming -m cc
  skm update brainstorming -p
  skm update -m cc
  skm update -p ~/myproject/skills")]
    Update {
        /// Skill name to update; omit to update all skills in the target directory
        name: Option<String>,

        /// Model name whose registered path will be used (e.g. cc, cursor)
        #[arg(short = 'm', long, value_name = "MODEL")]
        model: Option<String>,

        /// Target directory; omit PATH to use current directory
        #[arg(short = 'p', long = "project", num_args = 0..=1, value_name = "PATH",
              default_missing_value = ".")]
        project: Option<PathBuf>,
    },

    /// Manage skm configuration and model mappings
    #[command(after_help = "\
The built-in model 'cc' always maps to ~/.claude/skills/ and cannot be removed.

EXAMPLES:
  skm config show
  skm config set cursor ~/.cursor/skills
  skm config set zed /Users/me/.zed/skills
  skm config rm cursor
  skm config set default-target cursor
  skm config set default-target project")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Print the config file path and all current settings
    Show,

    /// Remove a user-defined model mapping
    #[command(alias = "rm")]
    Remove {
        /// Model name to remove (cannot remove built-in 'cc')
        name: String,
    },

    /// Set a config value or model path
    #[command(after_help = "\
SCALAR KEYS:
  default-target   Pre-selected target in interactive menu (model name or \"project\")

MODEL MAPPINGS:
  Any other key is treated as a model name mapped to the given path.

EXAMPLES:
  skm config set default-target cc
  skm config set default-target project
  skm config set cursor ~/.cursor/skills
  skm config set zed /Users/me/.zed/skills")]
    Set {
        /// 'default-target' or a model name
        key: String,
        /// Value (model name / \"project\") or skills directory path
        value: String,
    },
}
