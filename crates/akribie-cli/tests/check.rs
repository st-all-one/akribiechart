//! Contrato de borda de `akr check` e `akr schema` (processo real, sem mock).

mod common;

use common::{run, run_in, temp_dir, temp_file};

#[test]
fn valid_source_via_stdin_is_silent_and_succeeds() {
    let run = run(
        &["check"],
        Some("chart v1 \"x\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n"),
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(run.stdout, "");
}

#[test]
fn dash_reads_stdin() {
    let run = run(&["check", "-"], Some("chart v1 \"x\"\n"));
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
}

#[test]
fn invalid_value_exits_8_with_actionable_hint() {
    let run = run(
        &["check", "-"],
        Some("chart v1 \"x\"\nnode a \"A\" shape=cilinder\n"),
    );
    assert_eq!(run.code, 8);
    assert!(
        run.stdout.contains(":2:18: E003 invalid-value:"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("hint:") && run.stdout.contains("cylinder"),
        "{}",
        run.stdout
    );
}

#[test]
fn json_envelope_carries_diagnostics() {
    let run = run(
        &["check", "-", "--json"],
        Some("chart v1 \"x\"\nnode a \"A\" shape=cilinder\n"),
    );
    assert_eq!(run.code, 8);
    let value: serde_json::Value = serde_json::from_str(run.stdout.trim()).expect("JSON válido");
    assert_eq!(
        value.get("success").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        value.get("command").and_then(serde_json::Value::as_str),
        Some("check")
    );
    assert_eq!(
        value
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(serde_json::Value::as_str),
        Some("schema")
    );
    let diagnostics = value
        .get("data")
        .and_then(|data| data.get("diagnostics"))
        .and_then(serde_json::Value::as_array)
        .expect("diagnostics");
    let errors: Vec<&serde_json::Value> = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .get("severity")
                .and_then(serde_json::Value::as_str)
                == Some("error")
        })
        .collect();
    assert_eq!(errors.len(), 1, "{diagnostics:?}");
    assert_eq!(
        errors
            .first()
            .and_then(|d| d.get("code"))
            .and_then(serde_json::Value::as_str),
        Some("E003")
    );
    assert_eq!(
        errors
            .first()
            .and_then(|d| d.get("line"))
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
}

#[test]
fn params_source_replaces_positionals() {
    let run = run(
        &[
            "check",
            "--params",
            "{\"source\":\"chart v1 \\\"x\\\"\\n\"}",
        ],
        None,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
}

#[test]
fn params_plus_positional_is_usage_error() {
    let run = run(
        &[
            "check",
            "--params",
            "{\"source\":\"chart v1\"}",
            "/tmp/nao-existe.akrc",
        ],
        None,
    );
    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("--params"), "{}", run.stderr);
}

#[test]
fn params_unknown_key_lists_valid_keys() {
    let run = run(&["check", "--params", "{\"nope\":1}"], None);
    assert_eq!(run.code, 2);
    assert!(
        run.stderr.contains("path, source, strict"),
        "{}",
        run.stderr
    );
}

#[test]
fn params_path_and_source_are_exclusive() {
    let run = run(
        &[
            "check",
            "--params",
            "{\"path\":\"a.akrc\",\"source\":\"chart v1\"}",
        ],
        None,
    );
    assert_eq!(run.code, 2);
    assert!(
        run.stderr.contains("path") && run.stderr.contains("source"),
        "{}",
        run.stderr
    );
}

#[test]
fn missing_file_exits_3() {
    let path = std::env::temp_dir().join("akr-nao-existe-de-proposito.akrc");
    let run = run(&["check", path.to_str().unwrap_or_default()], None);
    assert_eq!(run.code, 3, "stderr: {}", run.stderr);
}

#[test]
fn file_input_works() {
    let path = temp_file("valid.akrc", "chart v1 \"x\"\nnode a\n");
    let run = run(&["check", path.to_str().unwrap_or_default()], None);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    drop(std::fs::remove_file(path));
}

#[test]
fn schema_exposes_closed_sets_and_codes() {
    let run = run(&["schema"], None);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    let value: serde_json::Value = serde_json::from_str(run.stdout.trim()).expect("JSON válido");
    let layout = value
        .get("statements")
        .and_then(|statements| statements.get("chart"))
        .and_then(|chart| chart.get("options"))
        .and_then(serde_json::Value::as_array)
        .and_then(|options| {
            options.iter().find(|option| {
                option.get("key").and_then(serde_json::Value::as_str) == Some("layout")
            })
        })
        .expect("layout no schema");
    let values: Vec<&str> = layout
        .get("values")
        .and_then(serde_json::Value::as_array)
        .map(|items| items.iter().filter_map(serde_json::Value::as_str).collect())
        .unwrap_or_default();
    assert_eq!(
        values,
        vec!["flow", "tree", "mindmap", "grid", "stack", "free"]
    );
    let codes = value
        .get("diagnostics")
        .and_then(serde_json::Value::as_array)
        .expect("diagnostics");
    assert_eq!(codes.len(), 16);
}

#[test]
fn broken_pipe_is_success() {
    assert_eq!(common::run_with_closed_stdout(&["schema"]), 0);
}

#[test]
fn strict_promotes_warnings_to_failure() {
    let source = "chart v1 \"x\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\nnode solto \"S\"\n";
    let plain = run(&["check"], Some(source));
    assert_eq!(plain.code, 0, "stderr: {}", plain.stderr);
    assert!(plain.stdout.contains("W002"), "stdout: {}", plain.stdout);
    let strict = run(&["check", "--strict"], Some(source));
    assert_eq!(strict.code, 8, "stdout: {}", strict.stdout);
    assert!(strict.stdout.contains("W002"), "stdout: {}", strict.stdout);
}

#[test]
fn config_from_project_directory_changes_layout() {
    let dir = std::env::temp_dir().join(format!("akr-config-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::write(dir.join("akribie.toml"), "layout = grid\n").expect("config");
    let file = dir.join("a.akrc");
    std::fs::write(&file, "chart v1 \"x\"\nnode a \"A\"\n").expect("akrc");
    let run = run(&["check", file.to_str().expect("utf-8")], None);
    assert_eq!(run.code, 0, "stdout: {} stderr: {}", run.stdout, run.stderr);
    assert_eq!(run.stdout, "", "layout=grid não deve emitir W002");
    std::fs::remove_dir_all(&dir).expect("cleanup");
}

#[test]
fn batch_reports_each_item_and_exits_zero_when_partially_valid() {
    let dir = temp_dir("batch-check");
    let batch = dir.join("lote.jsonl");
    std::fs::write(
        &batch,
        concat!(
            "{\"name\":\"um\",\"source\":\"chart v1 \\\"a\\\"\\nnode a \\\"A\\\"\\nnode b \\\"B\\\"\\nedge a -> b\\n\"}\n",
            "{\"name\":\"dois\",\"source\":\"chart v1 \\\"b\\\"\\nnode x \\\"X\\\"\\nnode y \\\"Y\\\"\\nedge x -> y\\n\"}\n",
            "{\"name\":\"ruim\",\"source\":\"chart v1 \\\"c\\\"\\nnode z shape=nope\\n\"}\n",
        ),
    )
    .expect("escreve lote");
    let run = run(&["check", "--batch", batch.to_str().expect("utf8")], None);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    let lines: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(lines.len(), 3, "{}", run.stdout);
    assert_eq!(lines[0], "um  ok  0 aviso(s)");
    assert!(lines[2].contains("  erro  E003 "), "{}", lines[2]);
}

#[test]
fn batch_fails_only_when_every_item_fails() {
    let dir = temp_dir("batch-check-all-bad");
    let batch = dir.join("lote.jsonl");
    std::fs::write(
        &batch,
        "{\"name\":\"ruim\",\"source\":\"chart v1 \\\"c\\\"\\nnode z shape=nope\\n\"}\n",
    )
    .expect("escreve lote");
    let run = run(&["check", "--batch", batch.to_str().expect("utf8")], None);
    assert_eq!(run.code, 8, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("ruim  erro  E003"), "{}", run.stdout);
}

#[test]
fn batch_item_needs_valid_name_and_single_source() {
    let dir = temp_dir("batch-check-name");
    let batch = dir.join("lote.jsonl");
    std::fs::write(
        &batch,
        concat!(
            "{\"source\":\"chart v1 \\\"a\\\"\\n\"}\n",
            "{\"name\":\"com espaço\",\"source\":\"chart v1 \\\"b\\\"\\n\"}\n",
            "{\"name\":\"dois\",\"source\":\"chart v1 \\\"c\\\"\\n\",\"path\":\"x.akrc\"}\n",
            "{\"name\":\"ok\",\"source\":\"chart v1 \\\"d\\\"\\nnode n \\\"N\\\"\\nnode m \\\"M\\\"\\nedge n -> m\\n\"}\n",
        ),
    )
    .expect("escreve lote");
    let run = run(&["check", "--batch", batch.to_str().expect("utf8")], None);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    let lines: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(lines.len(), 4, "{}", run.stdout);
    assert!(lines[0].contains("invalid_input") && lines[0].contains("sem `name`"));
    assert!(lines[1].contains("invalid_input"));
    assert!(lines[2].contains("invalid_input") && lines[2].contains("não ambos"));
    assert_eq!(lines[3], "ok  ok  0 aviso(s)");
}

#[test]
fn batch_strict_is_per_item() {
    let dir = temp_dir("batch-check-strict");
    let batch = dir.join("lote.jsonl");
    std::fs::write(
        &batch,
        concat!(
            "{\"name\":\"solto\",\"source\":\"chart v1 \\\"a\\\"\\nnode a \\\"A\\\"\\n\"}\n",
            "{\"name\":\"estrito\",\"source\":\"chart v1 \\\"b\\\"\\nnode b \\\"B\\\"\\n\",\"strict\":true}\n",
        ),
    )
    .expect("escreve lote");
    let run = run(&["check", "--batch", batch.to_str().expect("utf8")], None);
    assert_eq!(run.code, 0, "um item passa: {}", run.stderr);
    let lines: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(lines[0], "solto  ok  1 aviso(s)");
    assert!(lines[1].contains("estrito  erro  W002"), "{}", lines[1]);
}

#[test]
fn batch_json_has_counts_and_items() {
    let dir = temp_dir("batch-check-json");
    let batch = dir.join("lote.jsonl");
    std::fs::write(
        &batch,
        concat!(
            "{\"name\":\"ok\",\"source\":\"chart v1 \\\"a\\\"\\nnode a \\\"A\\\"\\nnode b \\\"B\\\"\\nedge a -> b\\n\"}\n",
            "{\"name\":\"bad\",\"source\":\"nada\\n\"}\n",
        ),
    )
    .expect("escreve lote");
    let run = run(
        &["check", "--batch", batch.to_str().expect("utf8"), "--json"],
        None,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("\"ok\":1"), "{}", run.stdout);
    assert!(run.stdout.contains("\"failed\":1"), "{}", run.stdout);
    assert!(run.stdout.contains("\"name\":\"bad\""), "{}", run.stdout);
}

#[test]
fn docs_lineage_resolves_relative_to_the_source() {
    let dir = temp_dir("docs-ok");
    std::fs::write(dir.join("guia.md"), "# Guia\nTexto qualquer.\n").expect("escreve doc");
    std::fs::write(
        dir.join("c.akrc"),
        "chart v1 \"x\" doc=\"guia.md#guia\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n",
    )
    .expect("escreve fonte");
    let run = run_in(&dir, &["check", "c.akrc", "--docs"], None);
    assert_eq!(run.code, 0, "stdout: {}", run.stdout);
    assert!(!run.stdout.contains("W004"), "{}", run.stdout);
}

#[test]
fn docs_lineage_reports_missing_document_or_anchor() {
    let dir = temp_dir("docs-missing");
    std::fs::write(
        dir.join("c.akrc"),
        "chart v1 \"x\" doc=\"naoexiste.md#guia\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n",
    )
    .expect("escreve fonte");
    let run = run_in(&dir, &["check", "c.akrc", "--docs"], None);
    assert_eq!(run.code, 0, "warning não bloqueia: {}", run.stdout);
    assert!(run.stdout.contains("W004"), "{}", run.stdout);
}

#[test]
fn quoted_label_may_contain_equals() {
    let run = run(
        &["check", "-"],
        Some("chart v1 \"T\"\nnode a \"user=admin\"\nnode b \"B\"\nedge a -> b\n"),
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
}
