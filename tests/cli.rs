use std::process::Command;

const KEY_A: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";

fn cli_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_miniscript-diagnostics"))
}

#[test]
fn json_flag_produces_valid_parseable_json_on_stdout() {
    let output = cli_command()
        .arg(format!("pk({KEY_A})"))
        .arg("--key")
        .arg(KEY_A)
        .arg("--json")
        .output()
        .expect("failed to run CLI binary");

    assert!(output.status.success());
    assert!(
        output.stderr.is_empty(),
        "stderr should be empty on success"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("--json stdout must be valid, parseable JSON with nothing else mixed in");
    assert_eq!(json["status"], "satisfied");
    assert_eq!(json["fragment"], format!("pk({KEY_A})"));
}

#[test]
fn default_output_is_human_readable_not_json() {
    let output = cli_command()
        .arg(format!("pk({KEY_A})"))
        .arg("--key")
        .arg(KEY_A)
        .output()
        .expect("failed to run CLI binary");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("SATISFIED"));
    assert!(stdout.contains("ROOT STATUS:"));
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_err());
}

#[test]
fn json_errors_still_go_to_stderr_not_mixed_into_stdout() {
    let output = cli_command()
        .arg("not_a_real_fragment(A)")
        .arg("--json")
        .output()
        .expect("failed to run CLI binary");

    assert!(!output.status.success());
    assert!(
        output.stdout.is_empty(),
        "stdout must stay empty on a parse error, even with --json"
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error:"));
}

#[test]
fn exit_code_zero_for_satisfied() {
    let output = cli_command()
        .arg(format!("pk({KEY_A})"))
        .arg("--key")
        .arg(KEY_A)
        .output()
        .expect("failed to run CLI binary");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn exit_code_two_for_unavailable() {
    let output = cli_command()
        .arg("older(144)")
        .arg("--elapsed-blocks")
        .arg("83")
        .output()
        .expect("failed to run CLI binary");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn exit_code_three_for_impossible() {
    let output = cli_command()
        .arg(format!("pk({KEY_A})"))
        .output()
        .expect("failed to run CLI binary");
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn exit_code_four_for_unsupported() {
    let output = cli_command()
        .arg(format!("andor(pk({KEY_A}),pk({KEY_A}),pk({KEY_A}))"))
        .output()
        .expect("failed to run CLI binary");
    assert_eq!(output.status.code(), Some(4));
}

#[test]
fn exit_code_one_for_invalid_input() {
    let output = cli_command()
        .arg("not_a_real_fragment(A)")
        .output()
        .expect("failed to run CLI binary");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn exit_code_reflects_root_status_in_json_mode_too() {
    let output = cli_command()
        .arg("older(144)")
        .arg("--json")
        .output()
        .expect("failed to run CLI binary");
    assert_eq!(output.status.code(), Some(2));
}
