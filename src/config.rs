use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::RegexSet;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub filter: FilterConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilterConfig {
    #[serde(default = "default_exact")]
    pub exact: Vec<String>,
    #[serde(default = "default_glob")]
    pub glob: Vec<String>,
    #[serde(default = "default_regex")]
    pub regex: Vec<String>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            exact: default_exact(),
            glob: default_glob(),
            regex: default_regex(),
        }
    }
}

pub struct FilterMatcher {
    exact: Vec<String>,
    glob: GlobSet,
    regex: RegexSet,
}

impl FilterMatcher {
    pub fn from_config(config: &FilterConfig) -> Result<Self, ConfigError> {
        let mut glob_builder = GlobSetBuilder::new();
        for pattern in &config.glob {
            glob_builder.add(
                Glob::new(pattern).map_err(|source| ConfigError::InvalidGlob {
                    pattern: pattern.clone(),
                    source,
                })?,
            );
        }

        let regex = RegexSet::new(&config.regex).map_err(ConfigError::InvalidRegex)?;

        Ok(Self {
            exact: config.exact.clone(),
            glob: glob_builder.build().map_err(ConfigError::BuildGlob)?,
            regex,
        })
    }

    pub fn matches(&self, name: &str) -> bool {
        self.exact.iter().any(|exact| exact == name)
            || self.glob.is_match(name)
            || self.regex.is_match(name)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    InvalidGlob {
        pattern: String,
        source: globset::Error,
    },
    BuildGlob(globset::Error),
    InvalidRegex(regex::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::CreateDir { path, source } => {
                write!(
                    f,
                    "failed to create config directory {}: {source}",
                    path.display()
                )
            }
            ConfigError::Write { path, source } => {
                write!(f, "failed to write config {}: {source}", path.display())
            }
            ConfigError::Read { path, source } => {
                write!(f, "failed to read config {}: {source}", path.display())
            }
            ConfigError::Parse { path, source } => {
                write!(f, "failed to parse config {}: {source}", path.display())
            }
            ConfigError::InvalidGlob { pattern, source } => {
                write!(f, "invalid glob pattern {pattern:?}: {source}")
            }
            ConfigError::BuildGlob(source) => write!(f, "failed to build glob matcher: {source}"),
            ConfigError::InvalidRegex(source) => write!(f, "invalid regex filter: {source}"),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn load_config(path: Option<PathBuf>) -> Result<Config, ConfigError> {
    match path {
        Some(path) => read_config_or_default(path),
        None => {
            let Some(path) = default_config_path() else {
                return Ok(Config::default());
            };
            read_or_create_default_config(path)
        }
    }
}

fn read_config_or_default(path: PathBuf) -> Result<Config, ConfigError> {
    if !path.exists() {
        return Ok(Config::default());
    }

    read_config(path)
}

fn read_or_create_default_config(path: PathBuf) -> Result<Config, ConfigError> {
    if !path.exists() {
        write_default_config(&path)?;
    }

    read_config(path)
}

fn write_default_config(path: &PathBuf) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    fs::write(path, default_config_toml()).map_err(|source| ConfigError::Write {
        path: path.clone(),
        source,
    })
}

fn read_config(path: PathBuf) -> Result<Config, ConfigError> {
    let raw = fs::read_to_string(&path).map_err(|source| ConfigError::Read {
        path: path.clone(),
        source,
    })?;

    toml::from_str(&raw).map_err(|source| ConfigError::Parse { path, source })
}

fn default_config_toml() -> &'static str {
    r#"[filter]
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
"#
}

#[cfg(test)]
fn load_default_config_at(path: PathBuf) -> Result<Config, ConfigError> {
    read_or_create_default_config(path)
}

#[cfg(test)]
fn load_explicit_config_at(path: PathBuf) -> Result<Config, ConfigError> {
    read_config_or_default(path)
}

fn default_config_path() -> Option<PathBuf> {
    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
        return Some(
            PathBuf::from(config_home)
                .join("maskrun")
                .join("config.toml"),
        );
    };

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = env::var_os("APPDATA") {
            return Some(PathBuf::from(appdata).join("maskrun").join("config.toml"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = env::var_os("HOME") {
            return Some(
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("maskrun")
                    .join("config.toml"),
            );
        }
    }

    env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("maskrun")
            .join("config.toml")
    })
}

fn default_exact() -> Vec<String> {
    ["API_KEY", "SECRET", "PASSWORD"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn default_glob() -> Vec<String> {
    ["*_KEY", "*_TOKEN", "*_SECRET", "*_PASSWORD"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn default_regex() -> Vec<String> {
    [r"(?i)^.*password.*$"]
        .into_iter()
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn matcher_supports_exact_glob_and_regex() {
        let config = FilterConfig {
            exact: vec!["SECRET".to_string()],
            glob: vec!["*_TOKEN".to_string()],
            regex: vec![r"(?i).*password.*".to_string()],
        };
        let matcher = FilterMatcher::from_config(&config).unwrap();

        assert!(matcher.matches("SECRET"));
        assert!(matcher.matches("API_TOKEN"));
        assert!(matcher.matches("db_password"));
        assert!(!matcher.matches("USERNAME"));
    }

    #[test]
    fn default_config_toml_is_parseable() {
        let config: Config = toml::from_str(default_config_toml()).unwrap();
        let matcher = FilterMatcher::from_config(&config.filter).unwrap();

        assert!(matcher.matches("API_KEY"));
        assert!(matcher.matches("SERVICE_TOKEN"));
        assert!(matcher.matches("db_password"));
    }

    #[test]
    fn creates_missing_default_config_file() {
        let root = temp_test_dir("creates-default-config");
        let path = root.join("maskrun").join("config.toml");

        let config = load_default_config_at(path.clone()).unwrap();

        assert!(path.exists());
        assert_eq!(config.filter.exact, default_exact());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn does_not_create_missing_explicit_config_file() {
        let root = temp_test_dir("skips-explicit-config");
        let path = root.join("custom").join("maskrun.toml");

        let config = load_explicit_config_at(path.clone()).unwrap();

        assert!(!path.exists());
        assert_eq!(config.filter.glob, default_glob());

        fs::remove_dir_all(root).ok();
    }

    fn temp_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("maskrun-{name}-{}-{nanos}", std::process::id()))
    }
}
