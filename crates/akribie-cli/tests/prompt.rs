//! Contrato de borda de `akr prompt`: texto estável, byte-idêntico por versão.

mod common;

use common::{read_golden, run};

#[test]
fn compact_prompt_is_byte_identical_and_matches_the_golden() {
    let first = run(&["prompt"], None);
    assert_eq!(first.code, 0, "stderr: {}", first.stderr);
    let second = run(&["prompt"], None);
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stdout, read_golden("prompt-compact.txt"));
    assert!(first.stdout.contains("exit"), "regra de exit codes");
    assert!(first.stdout.contains("Rewrite only the reported line"));
    assert!(first.stdout.contains("Accessibility is not optional"));
}

#[test]
fn long_appends_bnf_tables_and_theme_tokens() {
    let compact = run(&["prompt"], None);
    let long = run(&["prompt", "--long"], None);
    assert_eq!(long.code, 0, "stderr: {}", long.stderr);
    assert!(long.stdout.starts_with(&compact.stdout));
    assert!(long.stdout.contains("## BNF"));
    assert!(long.stdout.contains("## Keys per statement"));
    assert!(long.stdout.contains("## Theme tokens"));
    assert!(long.stdout.contains("slate: canvas="));
    assert!(long.stdout.contains("W007 low-contrast"));
    assert_eq!(long.stdout, read_golden("prompt-long.txt"));
}
