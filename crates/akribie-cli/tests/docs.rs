//! Paridade viva entre a ajuda dos verbos e `docs/usage/` (matriz T7).
//!
//! Uma flag nova sem documentação (ou documentada sem existir) quebra o CI.

mod common;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use common::run;

const VERBS: [(&str, &str); 8] = [
    ("check", "01-check.md"),
    ("fmt", "02-fmt.md"),
    ("render", "03-render.md"),
    ("import", "04-import.md"),
    ("diff", "05-diff.md"),
    ("init", "06-init-prompt.md"),
    ("prompt", "06-init-prompt.md"),
    ("schema", "07-schema.md"),
];

fn usage_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("usage")
}

/// Flags de forma longa (`--nome`) presentes em um texto.
fn flags(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for token in text.split(|character: char| {
        !(character.is_ascii_alphanumeric() || character == '-' || character == '_')
    }) {
        let is_flag = token.starts_with("--")
            && token.len() > 2
            && token[2..].chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            });
        let has_letter = token
            .chars()
            .any(|character| character.is_ascii_alphabetic());
        if is_flag && has_letter && token != "--help" {
            found.insert(token.to_string());
        }
    }
    found
}

#[test]
fn every_documented_usage_file_exists() {
    for verb in [
        "00-quickstart.md",
        "01-check.md",
        "02-fmt.md",
        "03-render.md",
        "04-import.md",
        "05-diff.md",
        "06-init-prompt.md",
        "07-schema.md",
    ] {
        let path = usage_dir().join(verb);
        assert!(path.is_file(), "falta {}", path.display());
    }
}

#[test]
fn help_and_usage_docs_agree_on_flags() {
    let mut all_flags: BTreeSet<String> = BTreeSet::new();
    let mut helps: Vec<(&'static str, &'static str, BTreeSet<String>)> = Vec::new();
    for (verb, file) in VERBS {
        let help = run(&[verb, "--help"], None);
        assert_eq!(help.code, 0, "`akr {verb} --help`: {}", help.stderr);
        let help_flags = flags(&help.stdout);
        all_flags.extend(help_flags.iter().cloned());
        helps.push((verb, file, help_flags));
    }
    for (verb, file, help_flags) in helps {
        let doc = std::fs::read_to_string(usage_dir().join(file)).expect("lê guia");
        let doc_flags = flags(&doc);
        for flag in &help_flags {
            assert!(
                doc_flags.contains(flag),
                "`{flag}` existe em `akr {verb} --help` mas não em docs/usage/{file}"
            );
        }
        for flag in &doc_flags {
            assert!(
                all_flags.contains(flag),
                "docs/usage/{file} cita `{flag}`, que nenhum verbo aceita"
            );
        }
    }
}
