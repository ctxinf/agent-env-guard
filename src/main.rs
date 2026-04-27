mod config;
mod mask;

use crate::config::{load_config, FilterMatcher};
use crate::mask::Masker;
use std::env;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::sync::Arc;
use std::thread;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("maskrun: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if wants_help(&args) {
        println!("{}", help_text());
        return Ok(ExitCode::SUCCESS);
    }

    let parsed = parse_args(args)?;

    let config = load_config(parsed.config_path)?;
    let matcher = FilterMatcher::from_config(&config.filter)?;
    let masker = Arc::new(Masker::from_environment(&matcher));

    if parsed.verbose {
        let verbose_lines = masker.verbose_lines();
        if verbose_lines.is_empty() {
            eprintln!("maskrun: no environment values matched the configured filters");
        }
        for line in verbose_lines {
            eprintln!("{line}");
        }
    }

    let mut child = Command::new(&parsed.command[0])
        .args(&parsed.command[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");
    let stdout_masker = Arc::clone(&masker);
    let stderr_masker = Arc::clone(&masker);

    let stdout_thread = thread::spawn(move || filter_stream(stdout, io::stdout(), stdout_masker));
    let stderr_thread = thread::spawn(move || filter_stream(stderr, io::stderr(), stderr_masker));

    let status = child.wait()?;
    stdout_thread
        .join()
        .map_err(|_| "stdout filter thread panicked")??;
    stderr_thread
        .join()
        .map_err(|_| "stderr filter thread panicked")??;

    Ok(exit_code(status.code()))
}

fn filter_stream<R, W>(mut reader: R, mut writer: W, masker: Arc<Masker>) -> io::Result<()>
where
    R: Read,
    W: Write,
{
    let mut input = Vec::new();
    reader.read_to_end(&mut input)?;
    let input = String::from_utf8_lossy(&input);
    writer.write_all(masker.mask_str(&input).as_bytes())?;
    writer.flush()
}

fn exit_code(code: Option<i32>) -> ExitCode {
    match code {
        Some(code) if (0..=255).contains(&code) => ExitCode::from(code as u8),
        Some(_) | None => ExitCode::from(1),
    }
}

fn wants_help(args: &[String]) -> bool {
    args.iter()
        .take_while(|arg| arg.as_str() != "--")
        .any(|arg| arg == "--help" || arg == "-h")
}

#[derive(Debug, Eq, PartialEq)]
struct ParsedArgs {
    config_path: Option<PathBuf>,
    verbose: bool,
    command: Vec<String>,
}

fn parse_args(args: Vec<String>) -> Result<ParsedArgs, String> {
    let mut config_path = None;
    let mut verbose = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => return Err(help_text()),
            "--verbose" | "-v" => {
                verbose = true;
            }
            "--config" => {
                index += 1;
                let Some(path) = args.get(index) else {
                    return Err("--config requires a path".to_string());
                };
                config_path = Some(PathBuf::from(path));
            }
            "--" => {
                let command = args[index + 1..].to_vec();
                if command.is_empty() {
                    return Err("missing command after --".to_string());
                }
                return Ok(ParsedArgs {
                    config_path,
                    verbose,
                    command,
                });
            }
            option if option.starts_with('-') => {
                return Err(format!("unknown option {option:?}\n\n{}", help_text()));
            }
            _ => {
                return Err(format!(
                    "expected -- before child command\n\n{}",
                    help_text()
                ));
            }
        }
        index += 1;
    }

    Err(format!("missing child command\n\n{}", help_text()))
}

fn help_text() -> String {
    "usage: maskrun [--verbose] [--config <path>] -- <raw_command> [raw_args...]\n\nmaskrun filters child stdout/stderr to reduce accidental secret exposure."
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_command_after_double_dash() {
        assert_eq!(
            parse_args(vec!["--".into(), "echo".into(), "hello".into()]).unwrap(),
            ParsedArgs {
                config_path: None,
                verbose: false,
                command: vec!["echo".into(), "hello".into()],
            }
        );
    }

    #[test]
    fn parses_config_before_command() {
        assert_eq!(
            parse_args(vec![
                "--config".into(),
                "maskrun.toml".into(),
                "--".into(),
                "env".into(),
            ])
            .unwrap(),
            ParsedArgs {
                config_path: Some(PathBuf::from("maskrun.toml")),
                verbose: false,
                command: vec!["env".into()],
            }
        );
    }

    #[test]
    fn parses_verbose_before_command() {
        assert_eq!(
            parse_args(vec!["--verbose".into(), "--".into(), "env".into()]).unwrap(),
            ParsedArgs {
                config_path: None,
                verbose: true,
                command: vec!["env".into()],
            }
        );
    }
}
