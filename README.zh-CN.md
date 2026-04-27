# agent-env-guard

当 OpenClaw、Hermes 这类 agent 执行命令时，保护你的 secrets 不以明文出现在输出里。

[English](./README.md)

`agent-env-guard` 提供 `maskrun`：一个给开发者和 coding agents 用的小型 CLI 包装器。

它允许命令正常使用环境变量，然后从 stdout 和 stderr 中打码命中的 secret value。

```bash
API_KEY=abc123xyz maskrun -- sh -c 'echo "key=$API_KEY"'
# key=a*******z
```

## ✨ 为什么用

- 🤖 Agent 安全：减少 tool call result、日志、transcript 里的 secret 泄露。
- 🧰 足够简单：只需要在原始命令前加一个前缀。

## 🚀 安装最新版

> 🌍 跨平台：Linux、macOS、Windows

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

### npm 项目

```bash
npm install @ctxinf/agent-env-guard@latest
```

## ⚡ 使用

### 1. 安装 `maskrun`

使用上面的任意一种安装方式。

### 2. 安装 agent skill

在你的 agent workspace 中执行：

```bash
npx skills add ctxinf/agent-env-guard
```

然后更新 `AGENTS.md` 或你的 agent 指令文件，要求模型强制遵守这个 skill。

> 这个 skill 会提示 agents：遇到可能泄露 secret 的命令时，使用 `maskrun --` 包一层。

### 3. 用 `maskrun` 执行命令

```bash
maskrun -- <command> [args...]
```

示例：

```bash
maskrun -- cargo test
maskrun -- npm run build
maskrun -- curl "https://api.example.com?key=${API_KEY}"
maskrun -- sh -c 'echo "$API_KEY"'
```

## 👀 它做什么

```bash
maskrun -- curl "https://api.example.com?key=${API_KEY}"
# 正常输出，但命中的 secret value 会被打码

maskrun -- bash -lc 'echo "$API_KEY"'
# a*******z
```

工作流程：

1. 读取环境变量。
2. 用配置匹配环境变量名。
3. 把命中的环境变量值在 stdout 和 stderr 中精确替换。
4. 保留子命令退出码。

## 🔒 安全边界

`maskrun` 只做输出脱敏，降低意外泄露风险。

它不是 sandbox、container、secret manager、network firewall 或 permission boundary。

子进程仍然可以读取环境变量、访问文件、使用网络。

## ⚙️ 配置

默认配置路径：

- Linux / Unix: `$XDG_CONFIG_HOME/maskrun/config.toml` 或 `$HOME/.config/maskrun/config.toml`
- macOS: `$HOME/Library/Application Support/maskrun/config.toml`
- Windows: `%APPDATA%\maskrun\config.toml`

首次运行会创建默认配置：

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

规则匹配的是环境变量名。命中后，对应环境变量值会被精确字符串替换。

使用自定义配置：

```bash
maskrun --config ./maskrun.toml -- env
```

显示命中的环境变量名，但不打印原始值：

```bash
maskrun --verbose -- sh -c 'echo "$API_KEY"'
```

## 🧪 快速测试

```bash
API_KEY=abc123xyz maskrun -- sh -c 'echo "$API_KEY"'
# a*******z
```

如果希望 `$API_KEY` 在子 shell 里展开，请使用单引号。

## 📦 Binary

```bash
maskrun --help
```

```text
usage: maskrun [--verbose] [--config <path>] -- <raw_command> [raw_args...]
```
