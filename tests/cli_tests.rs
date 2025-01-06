use std::process::Command;

#[test]
fn test_cli_output() {
    // Konfigurieren Sie den Prozess
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("start") // Hinzufügen des "start" Kommandos
        .arg("--duration")
        .arg("30")
        .arg("--description")
        .arg("Test session")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Duration: 30 minutes"));
    assert!(stdout.contains("Description: Test session"));

    let stderr = String::from_utf8_lossy(&output.stderr);

    let expected_stderr_start = "Finished `dev` profile [unoptimized + debuginfo]";
    assert!(
        stderr.trim().is_empty() || stderr.trim().starts_with(expected_stderr_start),
        "Stderr is not empty or doesn't start with expected content: {}",
        stderr
    );
}
