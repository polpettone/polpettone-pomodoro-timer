use std::process::{Command, Output};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

struct TestContext {
    _temp_dir: TempDir,
    config_path: PathBuf,
}

impl TestContext {
    fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        let session_dir = temp_dir.path().join("session");
        fs::create_dir_all(&session_dir).expect("Failed to create session directory");

        let config_content = format!(
            r#"
            [pomodoro_config]
            pomodoro_session_dir = "{}"
        "#,
            session_dir.display()
        );

        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, config_content).expect("Failed to write config");

        Self {
            _temp_dir: temp_dir,
            config_path,
        }
    }

    fn run(&self, command_args: &str) -> Output {
        let args = shell_words::split(command_args).expect("Failed to parse command arguments");

        Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--config")
            .arg(&self.config_path)
            .args(args)
            .output()
            .expect("Failed to execute cargo run")
    }
}

#[test]
fn test_cli_start_session() {
    let ctx = TestContext::new();
    let output = ctx.run("start --duration 30 --description 'Test session'");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Stdout: {}", stdout);
    println!("Stderr: {}", stderr);

    assert!(output.status.success(), "Command failed: {}", stderr);
    assert!(stdout.contains("Duration: 30 minutes"));
    assert!(stdout.contains("Description: Test session"));

    let expected_stderr_start = "Finished `dev` profile [unoptimized + debuginfo]";
    assert!(
        stderr.trim().is_empty() || stderr.trim().contains(expected_stderr_start),
        "Stderr contains unexpected errors: {}",
        stderr
    );
}

#[test]
fn test_cli_active_sessions() {
    let ctx = TestContext::new();
    // Start a session first
    ctx.run("start --duration 25 --description 'Work session'");
    
    // Check active sessions
    let output = ctx.run("active");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("Showing all sessions"));
    assert!(stdout.contains("Work session"));
}
