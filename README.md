# agent-env-guard

Protect your secrets from showing up in plain text when agents like OpenClaw or Hermes run commands.

[简体中文](./README.zh-CN.md)

`agent-env-guard` ships `maskrun`, a tiny CLI wrapper for developers and coding agents.

It lets commands use your normal environment, then masks matched secret values from stdout and stderr.

```bash
API_KEY=abc123xyz maskrun -- sh -c 'echo "key=$API_KEY"'
# key=a*******z
```

## ✨ Why

- 🤖 Agent-safe: reduce secret leaks in tool call results, logs, and transcripts.
- 🧰 Simple: add one prefix before the raw command.

## 🚀 Install Latest

> 🌍 Cross-platform: Linux, macOS, Windows

### Shell

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ctxinf/agent-env-guard/releases/latest/download/agent-env-guard-installer.sh | sh
```

### PowerShell

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ctxinf/agent-env-guard/releases/latest/download/agent-env-guard-installer.ps1 | iex"
```

### Homebrew

```bash
brew install ctxinf/tap/agent-env-guard
```

### npm Project

```bash
npm install @ctxinf/agent-env-guard@latest
```

## ⚡ Usage

### 1. Install `maskrun`

Use one of the install commands above.

### 2. Install the agent skill

From your agent workspace:

```bash
npx skills add ctxinf/agent-env-guard
```

Then update `AGENTS.md` or the other agent instruction files to force the model to follow the skill.

> The skill tells agents to wrap risky commands with `maskrun --`.

### 3. Run commands through `maskrun`

```bash
maskrun -- <command> [args...]
```

Examples:

```bash
maskrun -- cargo test
maskrun -- npm run build
maskrun -- curl "https://api.example.com?key=${API_KEY}"
maskrun -- sh -c 'echo "$API_KEY"'
```

## 👀 What It Does

```bash
maskrun -- curl "https://api.example.com?key=${API_KEY}"
# normal output, with matching secret values masked

maskrun -- bash -lc 'echo "$API_KEY"'
# a*******z
```

How it works:

1. Read environment variables.
2. Select variables whose names match the config.
3. Replace their exact values in stdout and stderr.
4. Return the child command exit code.

## 🔒 Security Boundary

`maskrun` filters command output to reduce accidental secret exposure.

It is not a sandbox, container, secret manager, network firewall, or permission boundary.

The child process can still read environment variables, access files, and use the network.

## ⚙️ Configuration

Default config path:

- Linux / Unix: `$XDG_CONFIG_HOME/maskrun/config.toml` or `$HOME/.config/maskrun/config.toml`
- macOS: `$HOME/Library/Application Support/maskrun/config.toml`
- Windows: `%APPDATA%\maskrun\config.toml`

Default config is created on first run:

```toml
[filter]
exact = [
  "API_KEY",
  "SECRET",
  "PASSWORD",
]

glob = [
  "*_KEY",
  "*_TOKEN",
  "*_SECRET",
  "*_PASSWORD",
]

regex = [
  "(?i)^.*password.*$",
]
```

Rules match environment variable names. Matched values are masked by exact string replacement.

Use a custom config:

```bash
maskrun --config ./maskrun.toml -- env
```

Show matched env names without printing raw values:

```bash
maskrun --verbose -- sh -c 'echo "$API_KEY"'
```

## 🧪 Quick Test

```bash
API_KEY=abc123xyz maskrun -- sh -c 'echo "$API_KEY"'
# a*******z
```

Use single quotes when secret expansion should happen inside the child shell.

## 📦 Binary

```bash
maskrun --help
```

```text
usage: maskrun [--verbose] [--config <path>] -- <raw_command> [raw_args...]
```
