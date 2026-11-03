//! Contrato de borda de `akr init`: idempotência e arquivos instalados.

mod common;

use common::{run_in, temp_dir};

#[test]
fn init_installs_files_and_the_second_run_changes_nothing() {
    let dir = temp_dir("init");
    let first = run_in(&dir, &["init"], None);
    assert_eq!(first.code, 0, "stderr: {}", first.stderr);
    assert!(
        first.stdout.contains("novo: akribie.toml"),
        "{}",
        first.stdout
    );
    assert!(first.stdout.contains("novo: AGENTS.md"), "{}", first.stdout);
    assert!(
        first.stdout.contains("novo: .agents/skills/akr/SKILL.md"),
        "{}",
        first.stdout
    );
    assert!(first.stdout.contains("próximos:"), "{}", first.stdout);
    for path in ["akribie.toml", "AGENTS.md", ".agents/skills/akr/SKILL.md"] {
        assert!(dir.join(path).is_file(), "{path}");
    }
    let skill = std::fs::read_to_string(dir.join(".agents/skills/akr/SKILL.md")).expect("skill");
    assert!(skill.contains("<!-- akribie:skill:version: 1 -->"));

    let agents_before = std::fs::read_to_string(dir.join("AGENTS.md")).expect("AGENTS.md");
    let second = run_in(&dir, &["init"], None);
    assert_eq!(second.code, 0, "stderr: {}", second.stderr);
    assert_eq!(
        second.stdout.matches("inalterado:").count(),
        3,
        "{}",
        second.stdout
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("AGENTS.md")).expect("AGENTS.md"),
        agents_before
    );
    assert_eq!(
        agents_before.matches("<!-- akribie:start -->").count(),
        1,
        "{agents_before}"
    );
}

#[test]
fn init_config_is_read_by_check() {
    let dir = temp_dir("init-config");
    let run = run_in(&dir, &["init"], None);
    assert_eq!(run.code, 0, "stderr: {}", run.stderr);
    std::fs::write(
        dir.join("chart.akrc"),
        "chart v1 \"x\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n",
    )
    .expect("escreve chart");
    let checked = run_in(&dir, &["check", "chart.akrc"], None);
    assert_eq!(
        checked.code, 0,
        "stdout: {} / stderr: {}",
        checked.stdout, checked.stderr
    );
}
