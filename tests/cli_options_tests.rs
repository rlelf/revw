use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn tmp_path(prefix: &str, ext: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "revw_{}_{}_{}.{}",
        prefix,
        std::process::id(),
        nanos,
        ext
    ))
}

fn run_cmd(args: &[String]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_revw"))
        .args(args)
        .output()
        .expect("failed to execute revw")
}

#[test]
fn inside_and_outside_flags_conflict() {
    let target = tmp_path("conflict_sections", "json");
    fs::write(&target, r#"{"outside":[],"inside":[]}"#).expect("failed to write target file");

    let output = run_cmd(&[
        "--stdout".to_string(),
        "--inside".to_string(),
        "--outside".to_string(),
        target.to_string_lossy().to_string(),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--inside"));
    assert!(stderr.contains("--outside"));
}

#[test]
fn output_format_flags_are_mutually_exclusive() {
    let target = tmp_path("conflict_formats", "json");
    fs::write(&target, r#"{"outside":[],"inside":[]}"#).expect("failed to write target file");

    let output = run_cmd(&[
        "--stdout".to_string(),
        "--markdown".to_string(),
        "--json".to_string(),
        target.to_string_lossy().to_string(),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--markdown"));
    assert!(stderr.contains("--json"));
}

#[test]
fn append_requires_input() {
    let target = tmp_path("append_requires_input", "json");
    fs::write(&target, r#"{"outside":[],"inside":[]}"#).expect("failed to write target file");

    let output = run_cmd(&["--append".to_string(), target.to_string_lossy().to_string()]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--input"));
}

#[test]
fn input_conflicts_with_stdout_mode() {
    let target = tmp_path("input_conflict_target", "json");
    let input = tmp_path("input_conflict_input", "json");
    fs::write(&target, r#"{"outside":[],"inside":[]}"#).expect("failed to write target file");
    fs::write(&input, r#"{"outside":[],"inside":[]}"#).expect("failed to write input file");

    let output = run_cmd(&[
        "--input".to_string(),
        input.to_string_lossy().to_string(),
        "--stdout".to_string(),
        target.to_string_lossy().to_string(),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--input"));
    assert!(stderr.contains("--stdout"));
}

#[test]
fn input_toon_overwrites_json_target() {
    let target = tmp_path("toon_to_json_target", "json");
    let input = tmp_path("toon_to_json_input", "toon");
    fs::write(&target, r#"{"outside":[],"inside":[]}"#).expect("failed to write target file");
    fs::write(
        &input,
        "outside[1]{name,context,url,percentage}:\n  \"Example\",\"Desc\",https://example.com,88\n\ninside[1]{date,context}:\n  \"2025-01-01 00:00:00\",\"Note\"\n",
    )
    .expect("failed to write input file");

    let output = run_cmd(&[
        "--input".to_string(),
        input.to_string_lossy().to_string(),
        target.to_string_lossy().to_string(),
    ]);

    assert!(output.status.success());
    let saved = fs::read_to_string(&target).expect("failed to read target file");
    assert!(saved.contains("\"Example\""));
    assert!(saved.contains("\"Note\""));
}

#[test]
fn input_toon_overwrites_markdown_target() {
    let target = tmp_path("toon_to_markdown_target", "md");
    let input = tmp_path("toon_to_markdown_input", "toon");
    fs::write(&target, "## OUTSIDE\n\n## INSIDE\n").expect("failed to write target file");
    fs::write(
        &input,
        "outside[1]{name,context,url,percentage}:\n  \"Example\",\"Desc\",https://example.com,88\n\ninside[1]{date,context}:\n  \"2025-01-01 00:00:00\",\"Note\"\n",
    )
    .expect("failed to write input file");

    let output = run_cmd(&[
        "--input".to_string(),
        input.to_string_lossy().to_string(),
        target.to_string_lossy().to_string(),
    ]);

    assert!(output.status.success());
    let saved = fs::read_to_string(&target).expect("failed to read target file");
    assert!(saved.contains("## OUTSIDE"));
    assert!(saved.contains("### Example"));
    assert!(saved.contains("## INSIDE"));
}
