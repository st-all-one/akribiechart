//! Contrato de `akr fmt`: forma canônica, idempotência e `--check`/`--write`.

mod common;

use common::{golden, read_golden, run, temp_file};

#[test]
fn unordered_source_becomes_canonical() {
    let input = golden("fmt_canonical.akrc");
    let expected = read_golden("fmt_expected.akrc");
    let result = run(&["fmt", input.to_str().unwrap()], None);
    assert_eq!(result.code, 0, "stderr: {}", result.stderr);
    assert_eq!(result.stdout, expected);
}

#[test]
fn canonical_form_is_idempotent() {
    let expected = read_golden("fmt_expected.akrc");
    let once = run(&["fmt", "-"], Some(&expected));
    assert_eq!(once.code, 0, "stderr: {}", once.stderr);
    let twice = run(&["fmt", "-"], Some(&once.stdout));
    assert_eq!(twice.stdout, once.stdout);
    assert_eq!(twice.stdout, expected);
}

#[test]
fn check_accepts_canonical_and_rejects_divergence() {
    let canonical = golden("fmt_expected.akrc");
    let ok = run(&["fmt", "--check", canonical.to_str().unwrap()], None);
    assert_eq!(ok.code, 0, "stderr: {}", ok.stderr);
    assert_eq!(ok.stdout, "");

    let unordered = golden("fmt_canonical.akrc");
    let diverged = run(&["fmt", "--check", unordered.to_str().unwrap()], None);
    assert_eq!(diverged.code, 1);
    assert_eq!(diverged.stdout, "");
    assert!(
        diverged.stderr.contains("forma canônica"),
        "{}",
        diverged.stderr
    );
}

#[test]
fn write_rewrites_the_file_in_place() {
    let source = read_golden("fmt_canonical.akrc");
    let expected = read_golden("fmt_expected.akrc");
    let path = temp_file("write.akrc", &source);
    let result = run(&["fmt", "--write", path.to_str().unwrap()], None);
    assert_eq!(result.code, 0, "stderr: {}", result.stderr);
    assert_eq!(
        std::fs::read_to_string(&path).expect("lê arquivo"),
        expected
    );
    drop(std::fs::remove_file(path));
}

#[test]
fn write_without_a_file_is_usage_error() {
    let result = run(&["fmt", "--write"], Some("chart v1 \"x\"\n"));
    assert_eq!(result.code, 2);
    assert!(result.stderr.contains("--write"), "{}", result.stderr);
}

#[test]
fn write_and_check_are_exclusive() {
    let path = temp_file("exclusive.akrc", "chart v1\n");
    let result = run(&["fmt", "--write", "--check", path.to_str().unwrap()], None);
    assert_eq!(result.code, 2);
    drop(std::fs::remove_file(path));
}

#[test]
fn params_source_replaces_flags() {
    let result = run(&["fmt", "--params", "{\"source\":\"chart   v1\\n\"}"], None);
    assert_eq!(result.code, 0, "stderr: {}", result.stderr);
    assert_eq!(result.stdout, "chart v1\n");
}

#[test]
fn params_plus_positional_is_usage_error() {
    let result = run(
        &["fmt", "--params", "{\"source\":\"chart v1\"}", "x.akrc"],
        None,
    );
    assert_eq!(result.code, 2);
}

#[test]
fn invalid_source_reports_diagnostics_and_exits_8() {
    let result = run(
        &["fmt", "-"],
        Some("chart v1 \"x\"\nnode a shape=cilinder\n"),
    );
    assert_eq!(result.code, 8);
    assert!(result.stdout.contains("E003"), "{}", result.stdout);
    assert!(result.stdout.contains("hint:"), "{}", result.stdout);
}

#[test]
fn empty_document_stays_empty() {
    let result = run(&["fmt", "-"], Some(""));
    assert_eq!(result.code, 0, "stderr: {}", result.stderr);
    assert_eq!(result.stdout, "");
}

#[test]
fn comments_without_statement_stay_at_the_end() {
    let result = run(&["fmt", "-"], Some("chart v1 \"x\"\n# solto\n# outro\n"));
    assert_eq!(result.code, 0, "stderr: {}", result.stderr);
    assert_eq!(result.stdout, "chart v1 \"x\"\n\n# solto\n# outro\n");
}
