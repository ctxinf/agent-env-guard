---
name: use-maskrun
description: Use maskrun instead of executing shell commands directly when commands may print environment variables, API keys, tokens, passwords, secrets, credentials, CI variables, cloud credentials, .env values, or other sensitive values. Trigger this skill before running terminal commands, test commands, build scripts, curl/API calls, package manager scripts, debugging commands, or agent automation where stdout or stderr might contain secret environment variable values.
---

# Use Maskrun

## Core Rule

Run potentially sensitive commands through `maskrun --` instead of executing them directly.

Use this form:

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

## When To Wrap

Wrap commands when any of these are true:

- The command may print environment variables or configuration values.
- The command touches `.env`, credentials, tokens, API keys, cloud config, CI config, auth headers, or debug dumps.
- The command runs tests, build scripts, package scripts, setup scripts, or third-party CLIs that may echo environment state.
- The command sends API requests using credentials from the environment.
- The command is being run by an agent and the output may be saved in logs or transcripts.

## When Direct Execution Is Fine

Direct execution is usually fine for commands that only inspect local source files or repository metadata and do not run project code, read env files, or print environment values.

Examples:

```bash
rg "maskrun" src tests README.md
sed -n '1,120p' Cargo.toml
git diff -- src/main.rs
```

If unsure, use `maskrun --`.

## Command Handling

Keep the wrapped command unchanged after `--`.

Do:

```bash
maskrun -- bash -lc 'echo "$API_KEY"'
maskrun -- env
maskrun -- cargo test -- --nocapture
```

Do not rewrite the child command arguments to make masking work. `maskrun` filters stdout and stderr while preserving the child command's normal inherited environment and exit code.

## Quoting

Use normal shell quoting rules.

Prefer single quotes around child shell snippets when secret expansion should happen inside the child process:

```bash
maskrun -- sh -c 'echo "$API_KEY"'
```

Avoid this when the parent shell would expand the secret before `maskrun` starts:

```bash
maskrun -- sh -c "echo $API_KEY"
```

## Safety Boundary

Treat `maskrun` as output masking only.

It does not sandbox the child process, block network access, prevent file writes, manage credentials, or stop the child command from reading environment variables. It reduces accidental exposure in terminal output, logs, and agent transcripts.

## If Maskrun Is Missing

If `maskrun` is unavailable, first check whether the current repository provides it.

For this Rust project, build or run it with Cargo:

```bash
cargo build
cargo run -- -- <command> [args...]
```

When installed, prefer the shorter production form:

```bash
maskrun -- <command> [args...]
```
