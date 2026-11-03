//! Contrato de borda de `akr diff` (processo real, sem mock).

mod common;

use common::{git, run, run_in, temp_dir, temp_file};

const OLD: &str = "chart v1 \"T\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n";
const NEW: &str =
    "chart v1 \"T\"\nnode a \"A2\"\nnode b \"B\"\nnode c \"C\"\nedge a -> b\nedge b -> c\n";

#[test]
fn three_changes_are_three_lines_in_order() {
    let old = temp_file("old.akrc", OLD);
    let new = temp_file("new.akrc", NEW);
    let run = run(
        &[
            "diff",
            old.to_str().unwrap_or_default(),
            new.to_str().unwrap_or_default(),
        ],
        None,
    );
    assert_eq!(run.code, 1, "stderr: {}", run.stderr);
    let lines: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(lines.len(), 3, "{}", run.stdout);
    assert_eq!(lines[0], "+ edge b->c");
    assert_eq!(lines[1], "+ node c \"C\"");
    assert_eq!(lines[2], "~ node a label: A -> A2");
}

#[test]
fn identical_files_are_silent_and_succeed() {
    let left = temp_file("left.akrc", OLD);
    let right = temp_file("right.akrc", OLD);
    let run = run(
        &[
            "diff",
            left.to_str().unwrap_or_default(),
            right.to_str().unwrap_or_default(),
        ],
        None,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(run.stdout, "");
}

#[test]
fn json_groups_changes_by_kind() {
    let old = temp_file("old.akrc", OLD);
    let new = temp_file("new.akrc", NEW);
    let run = run(
        &[
            "diff",
            old.to_str().unwrap_or_default(),
            new.to_str().unwrap_or_default(),
            "--json",
        ],
        None,
    );
    assert_eq!(run.code, 1, "stderr: {}", run.stderr);
    let envelope: serde_json::Value = serde_json::from_str(&run.stdout).unwrap_or_default();
    assert_eq!(envelope["success"], true);
    assert_eq!(envelope["data"]["added"].as_array().map(Vec::len), Some(2));
    assert_eq!(
        envelope["data"]["removed"].as_array().map(Vec::len),
        Some(0)
    );
    assert_eq!(
        envelope["data"]["changed"].as_array().map(Vec::len),
        Some(1)
    );
}

#[test]
fn invalid_side_blocks_the_diff() {
    let old = temp_file("old.akrc", OLD);
    let bad = temp_file("bad.akrc", "chart v1 \"x\"\nnode a shape=nope\n");
    let run = run(
        &[
            "diff",
            old.to_str().unwrap_or_default(),
            bad.to_str().unwrap_or_default(),
        ],
        None,
    );
    assert_eq!(run.code, 8);
    assert!(run.stdout.contains("E003"), "stdout: {}", run.stdout);
    assert!(
        run.stdout.contains("bad.akrc:2:14: E003"),
        "stdout: {}",
        run.stdout
    );
}

#[test]
fn missing_positionals_are_usage_errors() {
    let run = run(&["diff"], None);
    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("OLD"), "stderr: {}", run.stderr);
}

#[test]
fn both_sides_from_stdin_is_a_usage_error() {
    let run = run(&["diff", "-", "-"], Some(OLD));
    assert_eq!(run.code, 2);
    assert!(
        run.stderr.contains("entrada padrão"),
        "stderr: {}",
        run.stderr
    );
}

#[test]
fn at_revision_compares_the_working_tree_with_the_commit() {
    let dir = temp_dir("diff-at");
    let chart = dir.join("chart.akrc");
    std::fs::write(
        &chart,
        "chart v1 \"T\" layout=free\nnode a \"Antes\" at=10,10\n",
    )
    .expect("escreve chart");
    git(&dir, &["init", "-q", "."]);
    git(&dir, &["config", "commit.gpgsign", "false"]);
    git(&dir, &["config", "user.email", "t@example.com"]);
    git(&dir, &["config", "user.name", "t"]);
    git(&dir, &["add", "chart.akrc"]);
    git(&dir, &["commit", "-qm", "um"]);
    std::fs::write(
        &chart,
        "chart v1 \"T\" layout=free\nnode a \"Depois\" at=10,10\n",
    )
    .expect("altera chart");
    let run = run_in(&dir, &["diff", "chart.akrc", "--at", "HEAD"], None);
    assert_eq!(run.code, 1, "stdout: {}", run.stdout);
    assert!(
        run.stdout.contains("~ node a label: Antes -> Depois"),
        "{}",
        run.stdout
    );
}

#[test]
fn at_with_unknown_revision_is_not_found() {
    let dir = temp_dir("diff-at-missing");
    std::fs::write(
        dir.join("chart.akrc"),
        "chart v1 \"T\" layout=free\nnode a \"A\" at=10,10\n",
    )
    .expect("escreve chart");
    git(&dir, &["init", "-q", "."]);
    let run = run_in(&dir, &["diff", "chart.akrc", "--at", "naoexiste"], None);
    assert_eq!(run.code, 3, "stderr: {}", run.stderr);
}

#[test]
fn params_at_compares_the_revision_with_the_working_tree() {
    let dir = temp_dir("diff-params-at");
    let chart = dir.join("chart.akrc");
    std::fs::write(
        &chart,
        "chart v1 \"T\" layout=free\nnode a \"Antes\" at=10,10\n",
    )
    .expect("escreve chart");
    git(&dir, &["init", "-q", "."]);
    git(&dir, &["config", "commit.gpgsign", "false"]);
    git(&dir, &["config", "user.email", "t@example.com"]);
    git(&dir, &["config", "user.name", "t"]);
    git(&dir, &["add", "chart.akrc"]);
    git(&dir, &["commit", "-qm", "um"]);
    std::fs::write(
        &chart,
        "chart v1 \"T\" layout=free\nnode a \"Depois\" at=10,10\n",
    )
    .expect("altera chart");
    let run = run_in(
        &dir,
        &[
            "diff",
            "--params",
            "{\"old\":\"chart.akrc\",\"at\":\"HEAD\"}",
        ],
        None,
    );
    assert_eq!(run.code, 1, "stdout: {}", run.stdout);
    assert!(
        run.stdout.contains("~ node a label: Antes -> Depois"),
        "{}",
        run.stdout
    );
}

#[test]
fn params_with_both_sides_and_at_is_usage_error() {
    let run = run(
        &[
            "diff",
            "--params",
            "{\"old\":\"a.akrc\",\"new\":\"b.akrc\",\"at\":\"HEAD\"}",
        ],
        None,
    );
    assert_eq!(run.code, 2, "stderr: {}", run.stderr);
}
