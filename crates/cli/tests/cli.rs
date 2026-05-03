use std::process::Command;

#[test]
fn validates_query_with_dialect() {
    let output = Command::new(env!("CARGO_BIN_EXE_capsa"))
        .args(["-q", "SELECT * FROM users", "-d", "postgres"])
        .output()
        .expect("failed to run capsa");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_single_line_array(&stdout);
}

#[test]
fn rejects_missing_query() {
    let output = Command::new(env!("CARGO_BIN_EXE_capsa"))
        .args(["-d", "postgres"])
        .output()
        .expect("failed to run capsa");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage: capsa"));
}

#[test]
fn rejects_missing_dialect_value() {
    let output = Command::new(env!("CARGO_BIN_EXE_capsa"))
        .args(["-q", "SELECT * FROM users", "-d"])
        .output()
        .expect("failed to run capsa");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage: capsa"));
}

#[test]
fn validates_query_without_dialect() {
    let output = Command::new(env!("CARGO_BIN_EXE_capsa"))
        .args(["-q", "SELECT * FROM users"])
        .output()
        .expect("failed to run capsa");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_single_line_array(&stdout);
}

#[test]
fn outputs_validation_errors_on_one_line() {
    let output = Command::new(env!("CARGO_BIN_EXE_capsa"))
        .args(["-q", "select * from users"])
        .output()
        .expect("failed to run capsa");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_single_line_array(&stdout);
    assert!(stdout.contains("Lowercase keyword; uppercase keywords"));
}

fn assert_single_line_array(stdout: &str) {
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.starts_with('['));
    assert!(stdout.ends_with("]\n"));
}
