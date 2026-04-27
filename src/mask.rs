use crate::config::FilterMatcher;
use std::collections::HashSet;
use std::env;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Secret {
    names: Vec<String>,
    value: String,
    mask: String,
}

#[derive(Debug, Clone)]
pub struct Masker {
    secrets: Vec<Secret>,
}

impl Masker {
    pub fn from_environment(matcher: &FilterMatcher) -> Self {
        let mut seen = HashSet::new();
        let mut secrets = Vec::new();

        for (name, value) in env::vars() {
            if !matcher.matches(&name) || value.is_empty() {
                continue;
            }

            if let Some(secret) = secrets
                .iter_mut()
                .find(|secret: &&mut Secret| secret.value == value)
            {
                secret.names.push(name);
            } else if seen.insert(value.clone()) {
                secrets.push(Secret {
                    names: vec![name],
                    mask: mask_value(&value),
                    value,
                });
            }
        }

        secrets.sort_by(|left, right| right.value.len().cmp(&left.value.len()));

        Self { secrets }
    }

    #[cfg(test)]
    pub fn from_values(values: impl IntoIterator<Item = String>) -> Self {
        let mut secrets: Vec<_> = values
            .into_iter()
            .filter(|value| !value.is_empty())
            .map(|value| Secret {
                names: Vec::new(),
                mask: mask_value(&value),
                value,
            })
            .collect();
        secrets.sort_by(|left, right| right.value.len().cmp(&left.value.len()));
        Self { secrets }
    }

    pub fn mask_str(&self, input: &str) -> String {
        let mut output = input.to_string();
        for secret in &self.secrets {
            output = output.replace(&secret.value, &secret.mask);
        }
        output
    }

    pub fn verbose_lines(&self) -> Vec<String> {
        self.secrets
            .iter()
            .flat_map(|secret| {
                let value_len = secret.value.chars().count();
                secret.names.iter().map(move |name| {
                    format!(
                        "maskrun: matched env {name}={} (len={value_len})",
                        secret.mask
                    )
                })
            })
            .collect()
    }
}

fn mask_value(value: &str) -> String {
    let char_count = value.chars().count();
    if char_count <= 4 {
        return "*".repeat(char_count.max(6));
    }

    let mut chars = value.chars();
    let first = chars.next().unwrap_or('*');
    let last = value.chars().last().unwrap_or('*');
    let width = char_count.saturating_sub(2).max(6);

    format!("{first}{}{last}", "*".repeat(width))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_secret_values_and_keeps_edges() {
        let masker = Masker::from_values(["abc123xyz".to_string()]);

        assert_eq!(masker.mask_str("token=abc123xyz"), "token=a*******z");
    }

    #[test]
    fn masks_longer_values_first() {
        let masker = Masker::from_values(["abc123".to_string(), "abc123xyz".to_string()]);

        assert_eq!(masker.mask_str("abc123xyz abc123"), "a*******z a******3");
    }

    #[test]
    fn masks_short_values_without_revealing_edges() {
        let masker = Masker::from_values(["555".to_string()]);

        assert_eq!(masker.mask_str("key=555"), "key=******");
    }
}
