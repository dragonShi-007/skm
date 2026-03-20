use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug)]
struct ParsedUrl {
    owner: String,
    repo: String,
    branch: String,
    path: String,
    skill_name: String,
}

impl ParsedUrl {
    fn parse(url: &str) -> Result<Self> {
        let url = url.trim_end_matches('/');

        // Expected: https://github.com/owner/repo/tree/branch/path/to/skill
        let after_github = url
            .split("github.com/")
            .nth(1)
            .context("URL must be a GitHub URL (github.com/...)")?;

        let parts: Vec<&str> = after_github.split('/').collect();

        // Minimum: owner/repo/tree/branch/path
        if parts.len() < 5 {
            anyhow::bail!(
                "URL must point to a file or directory on GitHub.\n\
                Expected format: https://github.com/owner/repo/tree/branch/path/to/skill"
            );
        }

        if parts[2] != "tree" && parts[2] != "blob" {
            anyhow::bail!(
                "URL must be a tree (directory) or blob (file) URL.\n\
                Expected format: https://github.com/owner/repo/tree/branch/path/to/skill"
            );
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        let branch = parts[3].to_string();
        let path = parts[4..].join("/");
        let skill_name = parts.last().unwrap().to_string();

        Ok(ParsedUrl {
            owner,
            repo,
            branch,
            path,
            skill_name,
        })
    }
}

#[derive(Deserialize)]
struct CommitSha {
    sha: String,
}

#[derive(Deserialize)]
struct GithubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    content_type: String,
    download_url: Option<String>,
}

struct FileEntry {
    download_url: String,
    relative_path: PathBuf,
}

struct Downloader {
    client: reqwest::Client,
    owner: String,
    repo: String,
    branch: String,
}

impl Downloader {
    fn new(owner: &str, repo: &str, branch: &str) -> Result<Self> {
        let client = reqwest::Client::builder().user_agent("skm/0.1.0").build()?;
        Ok(Self {
            client,
            owner: owner.to_string(),
            repo: repo.to_string(),
            branch: branch.to_string(),
        })
    }

    fn api_url(&self, path: &str) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
            self.owner, self.repo, path, self.branch
        )
    }

    async fn collect_files(
        &self,
        path: &str,
        root_prefix: &str,
        files: &mut Vec<FileEntry>,
    ) -> Result<()> {
        let api_url = self.api_url(path);
        let resp = self
            .client
            .get(&api_url)
            .send()
            .await
            .context("Failed to reach GitHub API")?;

        if !resp.status().is_success() {
            anyhow::bail!("GitHub API returned {}: {}", resp.status(), api_url);
        }

        let body: serde_json::Value = resp.json().await?;

        if body.is_array() {
            let items: Vec<GithubContent> = serde_json::from_value(body)?;
            for item in items {
                match item.content_type.as_str() {
                    "file" => {
                        if let Some(dl_url) = item.download_url {
                            let rel_str = item
                                .path
                                .strip_prefix(root_prefix)
                                .unwrap_or(&item.path)
                                .trim_start_matches('/');
                            let relative_path = if rel_str.is_empty() {
                                PathBuf::from(&item.name)
                            } else {
                                PathBuf::from(rel_str)
                            };
                            files.push(FileEntry {
                                download_url: dl_url,
                                relative_path,
                            });
                        }
                    }
                    "dir" => {
                        Box::pin(self.collect_files(&item.path, root_prefix, files)).await?;
                    }
                    _ => {}
                }
            }
        } else {
            let item: GithubContent = serde_json::from_value(body)?;
            if let Some(dl_url) = item.download_url {
                let rel_str = item
                    .path
                    .strip_prefix(root_prefix)
                    .unwrap_or(&item.path)
                    .trim_start_matches('/');
                let relative_path = if rel_str.is_empty() {
                    PathBuf::from(&item.name)
                } else {
                    PathBuf::from(rel_str)
                };
                files.push(FileEntry {
                    download_url: dl_url,
                    relative_path,
                });
            }
        }

        Ok(())
    }

    async fn latest_commit_sha(&self, path: &str) -> Result<String> {
        let api_url = format!(
            "https://api.github.com/repos/{}/{}/commits?path={}&sha={}&per_page=1",
            self.owner, self.repo, path, self.branch
        );
        let resp = self
            .client
            .get(&api_url)
            .send()
            .await
            .context("Failed to reach GitHub API")?;
        if !resp.status().is_success() {
            anyhow::bail!("GitHub API returned {}: {}", resp.status(), api_url);
        }
        let commits: Vec<CommitSha> = resp.json().await?;
        commits
            .into_iter()
            .next()
            .map(|c| c.sha)
            .context("No commits found for this path")
    }

    async fn download_files(&self, files: Vec<FileEntry>, target_dir: &Path) -> Result<()> {
        let total = files.len() as u64;
        let pb = ProgressBar::new(total);
        pb.set_style(ProgressStyle::with_template(
            "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {wide_msg}",
        )?);

        for entry in files {
            let filename = entry
                .relative_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            pb.set_message(filename);

            let dest = target_dir.join(&entry.relative_path);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent).await?;
            }

            let bytes = self
                .client
                .get(&entry.download_url)
                .send()
                .await?
                .bytes()
                .await?;
            fs::write(&dest, bytes).await?;
            pb.inc(1);
        }

        pb.finish_and_clear();
        Ok(())
    }
}

pub async fn install(url: &str, target_dir: &Path) -> Result<()> {
    let parsed = ParsedUrl::parse(url)?;
    let skill_dir = target_dir.join(&parsed.skill_name);

    println!("Collecting file list for '{}'...", parsed.skill_name);

    let downloader = Downloader::new(&parsed.owner, &parsed.repo, &parsed.branch)?;

    let mut files = Vec::new();
    downloader
        .collect_files(&parsed.path, &parsed.path, &mut files)
        .await?;

    let sha = downloader.latest_commit_sha(&parsed.path).await?;

    downloader.download_files(files, &skill_dir).await?;

    // Persist URL and latest commit SHA so 'skm update' can check for changes
    fs::write(skill_dir.join(".skm-source"), format!("{}\n{}", url, sha)).await?;

    println!("✓ Installed '{}' successfully", parsed.skill_name);
    println!("  Location: {}", skill_dir.display());
    Ok(())
}

pub async fn update(name: Option<&str>, target_dir: &Path) -> Result<()> {
    match name {
        Some(n) => update_one(n, target_dir).await,
        None => update_all(target_dir).await,
    }
}

async fn update_one(name: &str, target_dir: &Path) -> Result<()> {
    let skill_dir = target_dir.join(name);
    let source_file = skill_dir.join(".skm-source");

    if !source_file.exists() {
        anyhow::bail!(
            "'{}' has no source record. Please reinstall with a full GitHub URL:\n  skm install https://github.com/OWNER/REPO/tree/BRANCH/path/to/skill",
            name
        );
    }

    let content = fs::read_to_string(&source_file).await?;
    let mut lines = content.lines();
    let url = lines.next().unwrap_or("").trim();
    let local_sha = lines.next().unwrap_or("").trim();

    let parsed = ParsedUrl::parse(url)?;
    let downloader = Downloader::new(&parsed.owner, &parsed.repo, &parsed.branch)?;
    let remote_sha = downloader.latest_commit_sha(&parsed.path).await?;

    if !local_sha.is_empty() && local_sha == remote_sha {
        println!("'{}' is already up to date.", name);
        return Ok(());
    }

    println!("Updating '{}'...", name);
    fs::remove_dir_all(&skill_dir).await?;
    install(url, target_dir).await
}

async fn update_all(target_dir: &Path) -> Result<()> {
    if !target_dir.exists() {
        println!(
            "No skills installed (directory not found: {})",
            target_dir.display()
        );
        return Ok(());
    }

    let mut entries = tokio::fs::read_dir(target_dir).await?;
    let mut names = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() && path.join(".skm-source").exists() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    if names.is_empty() {
        println!("No updatable skills found in {}", target_dir.display());
        return Ok(());
    }

    names.sort();
    let total = names.len();
    for name in names {
        update_one(&name, target_dir).await?;
    }
    println!("Updated {} skill(s).", total);
    Ok(())
}
