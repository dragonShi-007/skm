# skm — Skill Manager

**[English](#english) | [中文](#中文)**

---

<a name="english"></a>

## English

A command-line tool to install and manage AI skills for Claude Code and other AI assistants.

### Installation

#### From GitHub Releases

Download the pre-built binary for your platform from the [Releases](../../releases) page:

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `skm-macos-aarch64` |
| macOS (Intel) | `skm-macos-x86_64` |
| Linux (x86_64) | `skm-linux-x86_64` |
| Linux (ARM64) | `skm-linux-aarch64` |
| Windows (x86_64) | `skm-windows-x86_64.exe` |

Make it executable and move it to your PATH (Unix):

```bash
chmod +x skm-macos-aarch64
mv skm-macos-aarch64 /usr/local/bin/skm
```

#### From Source

```bash
cargo install --git https://github.com/<your-org>/skm --bin skm
```

Requires Rust 1.70+.

### Usage

```
skm <COMMAND> [OPTIONS]
```

#### Install a skill

```bash
# Install by full GitHub URL
skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming

# Install to Claude Code's global skills directory
skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming -m cc

# Install to the current project directory
skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming -p

# Install to a specific path
skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming -p ~/myproject/.claude/skills
```

**Target flags (pick at most one):**

| Flag | Effect |
|---|---|
| `-m MODEL` | Install to the path registered for MODEL (e.g. `cc`, `cursor`) |
| `-p` | Install to the current directory |
| `-p PATH` | Install to the specified path |
| _(none)_ | Show an interactive menu; default pre-selected by `skm config` |

#### List installed skills

```bash
skm list          # interactive menu to pick target
skm list -m cc    # list skills in Claude Code global directory
skm list -p       # list skills in current directory
```

#### Uninstall a skill

```bash
skm uninstall brainstorming -m cc
skm uninstall brainstorming -p ~/myproject/.claude/skills
```

#### Update skills

```bash
# Update a single skill
skm update brainstorming -m cc

# Update all skills in a target directory
skm update -m cc
skm update -p ~/myproject/.claude/skills
```

`skm update` checks the latest commit SHA against the installed version and skips skills that are already up to date.

#### Configuration

```bash
# Show current config and all registered model paths
skm config show

# Set the default target (used when no -m/-p flag is given)
skm config set default-target cc
skm config set default-target project

# Register a path for another AI assistant
skm config set cursor ~/.cursor/skills
skm config set zed /Users/me/.zed/skills

# Remove a user-defined model mapping
skm config rm cursor
```

Config is stored at `~/.config/skm/config.toml`.

### Models

A **model** is a short name mapped to a skills directory path. The built-in model `cc` always points to `~/.claude/skills/` and cannot be removed. You can register paths for other AI assistants (e.g. Cursor, Zed) with `skm config set`.

### How It Works

Skills are fetched directly from GitHub using the GitHub Contents API. No authentication is required for public repositories. Each installed skill records its source URL and commit SHA in a `.skm-source` file so `skm update` can detect upstream changes.

---

<a name="中文"></a>

## 中文

一个用于安装和管理 AI 技能（Skill）的命令行工具，支持 Claude Code 及其他 AI 助手。

### 安装

#### 从 GitHub Releases 下载

在 [Releases](../../releases) 页面下载对应平台的预编译二进制文件：

| 平台 | 文件名 |
|---|---|
| macOS（Apple Silicon）| `skm-macos-aarch64` |
| macOS（Intel）| `skm-macos-x86_64` |
| Linux（x86_64）| `skm-linux-x86_64` |
| Linux（ARM64）| `skm-linux-aarch64` |
| Windows（x86_64）| `skm-windows-x86_64.exe` |

Unix 系统下赋予执行权限并移动到 PATH：

```bash
chmod +x skm-macos-aarch64
mv skm-macos-aarch64 /usr/local/bin/skm
```

#### 从源码编译

```bash
cargo install --git https://github.com/<your-org>/skm --bin skm
```

需要 Rust 1.70+。

### 使用方法

```
skm <命令> [选项]
```

#### 安装技能

```bash
# 通过完整 GitHub URL 安装
skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming

# 安装到 Claude Code 全局技能目录
skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming -m cc

# 安装到当前项目目录
skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming -p

# 安装到指定路径
skm install https://github.com/obra/superpowers/tree/main/skills/brainstorming -p ~/myproject/.claude/skills
```

**目标位置标志（最多选一个）：**

| 标志 | 说明 |
|---|---|
| `-m MODEL` | 安装到 MODEL 对应的已注册路径（如 `cc`、`cursor`）|
| `-p` | 安装到当前目录 |
| `-p PATH` | 安装到指定路径 |
| _（不填）_ | 弹出交互式菜单，默认选项由 `skm config` 决定 |

#### 列出已安装技能

```bash
skm list          # 交互式菜单选择目标
skm list -m cc    # 列出 Claude Code 全局目录中的技能
skm list -p       # 列出当前目录中的技能
```

#### 卸载技能

```bash
skm uninstall brainstorming -m cc
skm uninstall brainstorming -p ~/myproject/.claude/skills
```

#### 更新技能

```bash
# 更新单个技能
skm update brainstorming -m cc

# 更新目标目录中的所有技能
skm update -m cc
skm update -p ~/myproject/.claude/skills
```

`skm update` 会对比本地与远程的 commit SHA，已是最新版本的技能会自动跳过。

#### 配置管理

```bash
# 查看当前配置和所有已注册路径
skm config show

# 设置默认目标（不传 -m/-p 时使用）
skm config set default-target cc
skm config set default-target project

# 为其他 AI 助手注册路径
skm config set cursor ~/.cursor/skills
skm config set zed /Users/me/.zed/skills

# 删除用户自定义的 model 映射
skm config rm cursor
```

配置文件路径：`~/.config/skm/config.toml`。

### Model 说明

**Model** 是一个短名称到技能目录路径的映射。内置 model `cc` 始终指向 `~/.claude/skills/`，不可删除。可通过 `skm config set` 为其他 AI 助手（如 Cursor、Zed）注册路径。

### 工作原理

技能文件通过 GitHub Contents API 直接从 GitHub 拉取，公开仓库无需认证。每个已安装的技能会在目录下生成 `.skm-source` 文件，记录来源 URL 和 commit SHA，供 `skm update` 检测是否有新版本。
