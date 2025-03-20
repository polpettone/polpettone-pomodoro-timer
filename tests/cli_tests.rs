use std::process::Command;

use std::fs;

#[test]
fn test_cli_output() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let session_dir = temp_dir.path().join("session");
    fs::create_dir_all(&session_dir).expect("Failed to create pomodoro directory");

    // Verwende `format!` für die String-Interpolation
    let config_content = format!(
        r#"
        [pomodoro_config]
        pomodoro_session_dir = "{}"
    "#,
        session_dir.display()
    );

    let config_path = temp_dir.path().join("pomodoro").join("config.toml");
    fs::create_dir_all(config_path.parent().unwrap()).expect("Failed to create config directory");
    fs::write(config_path.clone(), config_content).expect("Failed to write config");

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--config") // Verwenden Sie das Config-Argument
        .arg(config_path)
        .arg("start") // Hinzufügen des "start" Kommandos
        .arg("--duration")
        .arg("30")
        .arg("--description")
        .arg("Test session")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("Stdout: {}", stdout);
    println!("Stderr: {}", stderr);

    assert!(output.status.success());

    assert!(stdout.contains("Duration: 30 minutes"));
    assert!(stdout.contains("Description: Test session"));

    let expected_stderr_start = "Finished `dev` profile [unoptimized + debuginfo]";
    assert!(
        stderr.trim().is_empty() || stderr.trim().starts_with(expected_stderr_start),
        "Stderr is not empty or doesn't start with expected content: {}",
        stderr
    );
}
