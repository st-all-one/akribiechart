//! Contrato de borda de `akr render` (processo real, sem mock).

mod common;

use common::{git, run, run_in, temp_dir, temp_file};

fn png_width(bytes: &[u8]) -> Option<u32> {
    let header: [u8; 4] = bytes.get(16..20)?.try_into().ok()?;
    Some(u32::from_be_bytes(header))
}

const SAMPLE: &str = "chart v1 \"Retry\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b \"x\"\n";

#[test]
fn svg_goes_to_stdout_with_accessible_marks() {
    let run = run(&["render"], Some(SAMPLE));
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.starts_with("<svg "), "{}", run.stdout);
    assert!(run.stdout.contains("role=\"img\""));
    assert!(run.stdout.contains("<title id=\"akr-t\">Retry</title>"));
    assert!(run.stdout.contains("<desc id=\"akr-d\">"));
}

#[test]
fn two_renders_are_byte_identical() {
    let first = run(&["render"], Some(SAMPLE));
    let second = run(&["render"], Some(SAMPLE));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn svg_with_out_writes_the_file_and_stays_silent() {
    let path = temp_file("out.akrc", SAMPLE);
    let out = temp_file("out.svg", "");
    let run = run(
        &[
            "render",
            path.to_str().expect("utf-8"),
            "--out",
            out.to_str().expect("utf-8"),
        ],
        None,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(run.stdout, "");
    let written = std::fs::read_to_string(&out).expect("svg");
    assert!(written.contains("<svg "));
}

#[test]
fn png_scales_and_requires_out() {
    let missing = run(&["render", "--format", "png"], Some(SAMPLE));
    assert_eq!(missing.code, 2, "stdout: {}", missing.stdout);
    let out = temp_file("diagram.png", "");
    let run = run(
        &[
            "render",
            "-",
            "--format",
            "png",
            "--scale",
            "2",
            "--out",
            out.to_str().expect("utf-8"),
        ],
        Some(SAMPLE),
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    let bytes = std::fs::read(&out).expect("png");
    assert_eq!(
        &bytes[..8],
        &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]
    );
    let width = png_width(&bytes);
    assert!(width.is_some_and(|value| value > 300), "largura: {width:?}");
}

#[test]
fn invalid_source_is_not_rendered() {
    let run = run(&["render"], Some("chart v1 \"x\"\nnode a shape=nope\n"));
    assert_eq!(run.code, 8);
    // Diagnóstico é dado: vai para stdout, não para stderr.
    assert!(run.stdout.contains("E003"), "stdout: {}", run.stdout);
    assert!(
        run.stdout.starts_with("<stdin>:2:"),
        "stdout: {}",
        run.stdout
    );
    assert_eq!(run.stderr, "");
}

#[test]
fn unknown_format_is_usage_error() {
    let run = run(&["render", "--format", "gif"], Some(SAMPLE));
    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("gif"), "stderr: {}", run.stderr);
}

#[test]
fn json_reports_the_written_size() {
    let out = temp_file("meta.png", "");
    let run = run(
        &[
            "render",
            "-",
            "--format",
            "png",
            "--out",
            out.to_str().unwrap_or_default(),
            "--json",
        ],
        Some(SAMPLE),
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    let size = std::fs::metadata(&out).map_or(0, |meta| meta.len());
    let expected = format!("\"bytes\":{size}");
    assert!(run.stdout.contains(&expected), "stdout: {}", run.stdout);
    assert!(run.stdout.contains("\"path\":\""), "stdout: {}", run.stdout);
    assert!(run.stdout.contains("\"width\":"), "stdout: {}", run.stdout);
    assert!(run.stdout.contains("\"height\":"), "stdout: {}", run.stdout);
    assert!(
        run.stdout.contains("\"dry_run\":false"),
        "stdout: {}",
        run.stdout
    );
}

#[test]
fn scene_format_is_json_on_stdout() {
    let run = run(&["render", "--format", "scene"], Some(SAMPLE));
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    let value: serde_json::Value = serde_json::from_str(&run.stdout).unwrap_or_default();
    assert_eq!(value["layout"], "flow");
    assert_eq!(value["nodes"].as_array().map(Vec::len), Some(2));
    assert!(run.stdout.ends_with("}\n"));
}

#[test]
fn batch_writes_only_valid_items_and_stays_zero_when_some_succeed() {
    let dir = temp_dir("batch-render");
    let out = dir.join("saida");
    std::fs::create_dir_all(&out).expect("cria out-dir");
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
    let run = run(
        &[
            "render",
            "--batch",
            batch.to_str().expect("utf8"),
            "--out-dir",
            out.to_str().expect("utf8"),
        ],
        None,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(out.join("um.svg").is_file());
    assert!(out.join("dois.svg").is_file());
    assert!(!out.join("ruim.svg").exists());
    let lines: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(lines.len(), 3, "{}", run.stdout);
    assert!(lines[0].contains("  ok  "), "{}", lines[0]);
    assert!(
        lines[2].contains("  erro  ") && lines[2].contains("E003"),
        "{}",
        lines[2]
    );
}

#[test]
fn batch_honors_per_item_format_and_scale() {
    let dir = temp_dir("batch-render-format");
    let out = dir.join("saida");
    std::fs::create_dir_all(&out).expect("cria out-dir");
    let batch = dir.join("lote.jsonl");
    let target = dir.join("alvo.png");
    let target = serde_json::to_string(target.to_str().expect("utf8")).expect("json");
    std::fs::write(
        &batch,
        format!(
            concat!(
                "{{\"name\":\"um\",\"source\":\"chart v1 \\\"a\\\"\\nnode a \\\"A\\\"\\nnode b \\\"B\\\"\\nedge a -> b\\n\",\"format\":\"png\",\"scale\":1}}\n",
                "{{\"name\":\"dois\",\"source\":\"chart v1 \\\"b\\\"\\nnode x \\\"X\\\"\\nnode y \\\"Y\\\"\\nedge x -> y\\n\",\"out\":{},\"format\":\"png\",\"scale\":1}}\n",
            ),
            target
        ),
    )
    .expect("escreve lote");
    let run = run(
        &[
            "render",
            "--batch",
            batch.to_str().expect("utf8"),
            "--out-dir",
            out.to_str().expect("utf8"),
        ],
        None,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    let png = std::fs::read(out.join("um.png")).expect("um.png");
    assert_eq!(png.get(1..4), Some(&b"PNG"[..]), "assinatura PNG");
    assert!(!out.join("um.svg").exists());
    assert!(
        dir.join("alvo.png").is_file(),
        "out do item vence a derivação"
    );
}

#[test]
fn batch_requires_existing_out_dir_and_rejects_empty_or_conflicting_use() {
    let dir = temp_dir("batch-render-errors");
    let batch = dir.join("lote.jsonl");
    std::fs::write(
        &batch,
        "{\"name\":\"a\",\"source\":\"chart v1 \\\"a\\\"\\n\"}\n",
    )
    .expect("escreve lote");
    let missing = run(
        &[
            "render",
            "--batch",
            batch.to_str().expect("utf8"),
            "--out-dir",
            dir.join("nao-existe").to_str().expect("utf8"),
        ],
        None,
    );
    assert_eq!(missing.code, 3, "stderr: {}", missing.stderr);

    let empty = dir.join("vazio.jsonl");
    std::fs::write(&empty, "\n\n").expect("escreve lote vazio");
    let empty_run = run(&["render", "--batch", empty.to_str().expect("utf8")], None);
    assert_eq!(empty_run.code, 2, "stderr: {}", empty_run.stderr);

    let conflict = run(
        &[
            "render",
            "--batch",
            batch.to_str().expect("utf8"),
            "--params",
            "{}",
        ],
        None,
    );
    assert_eq!(conflict.code, 2, "stderr: {}", conflict.stderr);
}

#[test]
fn batch_json_counts_items_and_reports_targets() {
    let dir = temp_dir("batch-render-json");
    let out = dir.join("saida");
    std::fs::create_dir_all(&out).expect("cria out-dir");
    let batch = dir.join("lote.jsonl");
    std::fs::write(&batch, "{\"name\":\"a\",\"source\":\"chart v1 \\\"a\\\"\\nnode n \\\"N\\\"\\nnode m \\\"M\\\"\\nedge n -> m\\n\"}\n")
        .expect("escreve lote");
    let run = run(
        &[
            "render",
            "--batch",
            batch.to_str().expect("utf8"),
            "--out-dir",
            out.to_str().expect("utf8"),
            "--json",
        ],
        None,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("\"ok\":1"), "{}", run.stdout);
    assert!(run.stdout.contains("\"failed\":0"), "{}", run.stdout);
    assert!(run.stdout.contains("\"ok\":true"), "{}", run.stdout);
}

#[test]
fn batch_dry_run_writes_nothing() {
    let dir = temp_dir("batch-render-dry");
    let out = dir.join("saida");
    std::fs::create_dir_all(&out).expect("cria out-dir");
    let batch = dir.join("lote.jsonl");
    std::fs::write(&batch, "{\"name\":\"a\",\"source\":\"chart v1 \\\"a\\\"\\nnode n \\\"N\\\"\\nnode m \\\"M\\\"\\nedge n -> m\\n\",\"format\":\"png\"}\n")
        .expect("escreve lote");
    let run = run(
        &[
            "render",
            "--batch",
            batch.to_str().expect("utf8"),
            "--out-dir",
            out.to_str().expect("utf8"),
            "--dry-run",
        ],
        None,
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert_eq!(
        std::fs::read_dir(&out).expect("out-dir").count(),
        0,
        "dry-run não grava"
    );
    assert!(run.stdout.contains("  ok  "), "{}", run.stdout);
}

#[test]
fn dry_run_reports_bytes_without_writing() {
    let dir = temp_dir("dry-run");
    let target = dir.join("chart.png");
    let run = run(
        &[
            "render",
            "--format",
            "png",
            "--out",
            target.to_str().expect("utf8"),
            "--dry-run",
            "--json",
        ],
        Some(SAMPLE),
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(!target.exists(), "nada é escrito");
    assert!(run.stdout.contains("\"dry_run\":true"), "{}", run.stdout);
    assert!(run.stdout.contains("\"bytes\":"), "{}", run.stdout);
}

#[test]
fn excalidraw_export_is_json_with_order_keys() {
    let run = run(
        &["render", "-", "--format", "excalidraw"],
        Some("chart v1 \"x\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n"),
    );
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(
        run.stdout.contains("\"type\": \"excalidraw\""),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("\"index\": \"a0000V\""),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout.contains("\"endArrowhead\": \"arrow\""),
        "{}",
        run.stdout
    );
}

#[test]
fn at_revision_renders_the_committed_source() {
    let dir = temp_dir("render-at");
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
    let run = run_in(&dir, &["render", "chart.akrc", "--at", "HEAD"], None);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(
        run.stdout.contains("aria-label=\"Antes\""),
        "{}",
        run.stdout
    );
    assert!(!run.stdout.contains("Depois"));
}

/// Bloco `<g data-id="ID" …>…</g>` do SVG.
fn element_block(svg: &str, id: &str) -> String {
    let marker = format!("data-id=\"{id}\"");
    let Some(start) = svg.find(&marker) else {
        return String::new();
    };
    let end = svg[start..]
        .find("</g>")
        .map_or(svg.len(), |offset| start + offset);
    svg[start..end].to_string()
}

#[test]
fn clean_finish_and_sans_font_reach_the_svg() {
    let source =
        "chart v1 \"T\" sketch=off font=sans\nnode a \"A\"\nnode b \"B\"\nedge a -> b \"x\"\n";
    let run = run(&["render"], Some(source));
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(
        run.stdout
            .contains("font-family=\"Liberation Sans, Arial, Helvetica, sans-serif\""),
        "{}",
        run.stdout
    );
    let block = element_block(&run.stdout, "a");
    assert_eq!(block.matches("<path").count(), 1, "{block}");
    assert_eq!(block.matches('M').count(), 1, "{block}");
    assert!(
        block.contains('A'),
        "traço técnico usa arcos reais: {block}"
    );
}

#[test]
fn hand_finish_keeps_the_default_stack() {
    let source = "chart v1 \"T\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b \"x\"\n";
    let run = run(&["render"], Some(source));
    assert!(
        run.stdout
            .contains("font-family=\"Excalifont, Virgil, Comic Shanns, cursive\"")
    );
    let block = element_block(&run.stdout, "a");
    assert_eq!(block.matches('M').count(), 2, "duas passagens: {block}");
}

#[test]
fn edge_label_background_is_painted_after_the_line() {
    let source = "chart v1 \"x\" layout=free\nnode a \"A\" at=0,0\nnode b \"B\" at=300,0\nedge a -> b \"rotulo\"\n";
    let run = run(&["render"], Some(source));
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    let block = element_block(&run.stdout, "a-&gt;b#0");
    let path = block.find("<path d=").expect("traço");
    let rect = block.find("<rect ").expect("fundo do rótulo");
    let text = block.find("<text ").expect("texto");
    assert!(path < rect, "fundo precisa cobrir o traço: {block}");
    assert!(rect < text, "texto precisa ficar sobre o fundo: {block}");
}

#[test]
fn project_config_sets_finish_and_font() {
    let dir = temp_dir("render-config-finish");
    std::fs::write(dir.join("akribie.toml"), "sketch = off\nfont = sans\n").expect("config");
    std::fs::write(
        dir.join("c.akrc"),
        "chart v1 \"C\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n",
    )
    .expect("fonte");
    let run = run_in(&dir, &["render", "c.akrc"], None);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("Liberation Sans"));
    let block = element_block(&run.stdout, "a");
    assert_eq!(block.matches('M').count(), 1, "{block}");
}
