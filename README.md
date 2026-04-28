# agent-env-guard

Mask your secrets in bash tool_call results when using OpenClaw/Hermes/... .

[简体中文](./README.zh-CN.md)

```bash
# Before:
# The model thinks: "Why isn't this working? Let me check if API_KEY exists."
echo $API_KEY
sk-123454545677888 # secret exposed in a tool message

# After:
# The model thinks: "Why isn't this working? Let me check if API_KEY exists. The skill says to prefix commands with maskrun? OK."
maskrun -- echo $API_KEY
sk-1*************8
```

`agent-env-guard` ships `maskrun`, a tiny CLI wrapper for developers and coding agents.

It masks matched secret values from stdout and stderr.



## ✨ Feature (only one)

- 🤖 Agent-safe: mask secrets in exec outputs


## ⚡ Usage

### 1. Install `maskrun`

Use one of the install commands below.

### 2. Install the agent skill

From your agent workspace:

```bash
npx skills add ctxinf/agent-env-guard
```

Then update `AGENTS.md` or the other agent instruction files to force the model to follow the skill.

> The skill tells agents to wrap risky commands with `maskrun --`.


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
