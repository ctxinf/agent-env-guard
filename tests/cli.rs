use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn masks_stdout_and_preserves_exit_success() {
    let config_home = temp_config_home("stdout-success");
    let output = Command::new(env!("CARGO_BIN_EXE_maskrun"))
        .env("XDG_CONFIG_HOME", &config_home)
        .env("API_KEY", "abc123xyz")
        .args(command_args("echo $API_KEY"))
        .output()
        .unwrap();

    std::fs::remove_dir_all(config_home).unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "a*******z");
}

#[test]
fn masks_stderr_and_preserves_exit_failure() {
    let config_home = temp_config_home("stderr-failure");
    let output = Command::new(env!("CARGO_BIN_EXE_maskrun"))
        .env("XDG_CONFIG_HOME", &config_home)
        .env("API_KEY", "abc123xyz")
        .args(command_args("echo $API_KEY 1>&2; exit 7"))
        .output()
        .unwrap();

    std::fs::remove_dir_all(config_home).unwrap();

    assert_eq!(output.status.code(), Some(7));
    assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), "a*******z");
}

#[test]
fn verbose_prints_matched_keys_with_masked_values() {
    let config_home = temp_config_home("verbose");
    let output = Command::new(env!("CARGO_BIN_EXE_maskrun"))
        .env("XDG_CONFIG_HOME", &config_home)
        .env("API_KEY", "abc123xyz")
        .args(command_args_with_maskrun_args(&["--verbose"], "echo ok"))
        .output()
        .unwrap();

    std::fs::remove_dir_all(config_home).unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("matched env API_KEY=a*******z (len=9)"));
    assert!(!stderr.contains("abc123xyz"));
}

#[cfg(unix)]
fn command_args(script: &str) -> Vec<String> {
    vec!["--".into(), "sh".into(), "-c".into(), script.into()]
}

#[cfg(unix)]
fn command_args_with_maskrun_args(maskrun_args: &[&str], script: &str) -> Vec<String> {
    maskrun_args
        .iter()
        .map(|arg| (*arg).to_string())
        .chain(command_args(script))
        .collect()
}

#[cfg(windows)]
fn command_args(script: &str) -> Vec<String> {
    vec![
        "--".into(),
        "cmd".into(),
        "/C".into(),
        script
            .replace("$API_KEY", "%API_KEY%")
            .replace("1>&2", "1>&2")
            .into(),
    ]
}

#[cfg(windows)]
fn command_args_with_maskrun_args(maskrun_args: &[&str], script: &str) -> Vec<String> {
    maskrun_args
        .iter()
        .map(|arg| (*arg).to_string())
        .chain(command_args(script))
        .collect()
}

fn temp_config_home(name: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("maskrun-cli-{name}-{}-{nanos}", std::process::id()))
}
