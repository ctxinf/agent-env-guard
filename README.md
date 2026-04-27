# agent-env-guard

`maskrun` runs a child command and masks configured environment variable values
from the child's stdout and stderr.

It only filters output. It is not a sandbox, container, secret manager, network
firewall, or permission boundary.

```bash
maskrun -- curl "https://api.example.com?key=${API_KEY}"
maskrun -- bash -lc 'echo "$API_KEY"'
```

## Configuration

By default, `maskrun` reads and creates on first run:

- Linux and other Unix: `$XDG_CONFIG_HOME/maskrun/config.toml` or
  `$HOME/.config/maskrun/config.toml`
- macOS: `$HOME/Library/Application Support/maskrun/config.toml`
- Windows: `%APPDATA%\maskrun\config.toml`

You can also pass a config explicitly:

```bash
maskrun --config ./maskrun.toml -- env
```

Explicit `--config` paths are read if they exist. Missing explicit config files
are not created automatically.

Use `--verbose` before `--` to print which environment variable names matched
the filters. Values are still masked in verbose logs:

```bash
maskrun --verbose -- sh -c 'echo "$API_KEY"'
```

Example:

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
]

regex = [
  "(?i)^.*password.*$",
]
```

The filter rules match environment variable names. Matching variable values are
then replaced exactly in stdout and stderr. The child process receives the
normal inherited environment, and `maskrun` preserves the child exit code.

When testing from a shell, quote commands that should expand inside the child
process:

```bash
API_KEY=abc123xyz maskrun -- sh -c 'echo "$API_KEY"'
```

Without quoting, the parent shell expands `$API_KEY` before `maskrun` starts.
