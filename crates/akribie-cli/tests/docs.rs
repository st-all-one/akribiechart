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
    ("init", "06-init-prime.md"),
    ("prime", "06-init-prime.md"),
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

/// Verbos que o binário anuncia, lidos do próprio `--help` do clap.
///
/// A crate é binária (`[[bin]] name = "akr"`), então o teste de integração não alcança
/// `Cli::command()`: a lista é derivada da mesma fonte que o clap usa para gerar a ajuda. Uma
/// verbo novo sem guia, ou um guia de verbo que não existe, quebra o teste.
fn declared_verbs() -> BTreeSet<String> {
    let help = run(&["--help"], None);
    assert_eq!(help.code, 0, "`akr --help`: {}", help.stderr);
    let mut verbs = BTreeSet::new();
    let mut inside = false;
    for line in help.stdout.lines() {
        if line.starts_with("Commands:") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        let Some(name) = line.split_whitespace().next() else {
            continue;
        };
        let valid = name.chars().all(|character| character.is_ascii_lowercase());
        if valid && name != "help" {
            verbs.insert(name.to_string());
        }
    }
    assert!(!verbs.is_empty(), "`akr --help` não listou verbo nenhum");
    verbs
}

/// `akr <verbo>` tem seção própria na matriz de aceitação.
fn matrix_mentions(matrix: &str, verb: &str) -> bool {
    matrix.contains(&format!("## `akr {verb}`"))
}

fn matrix_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("plan")
        .join("02-matriz-aceitacao.md")
}

#[test]
fn every_subcommand_is_documented_and_covered_by_the_matrix() {
    let documented: BTreeSet<String> = VERBS.iter().map(|(verb, _)| (*verb).to_string()).collect();
    let declared = declared_verbs();
    assert_eq!(
        declared, documented,
        "todo verbo do binário precisa de guia em docs/usage/ e vice-versa"
    );
    let matrix = std::fs::read_to_string(matrix_path()).expect("lê matriz de aceitação");
    for (verb, file) in VERBS {
        assert!(
            usage_dir().join(file).is_file(),
            "`{verb}` mapeia para docs/usage/{file}, que não existe"
        );
        assert!(
            matrix_mentions(&matrix, verb),
            "`akr {verb}` não tem seção na matriz de aceitação"
        );
    }
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
        "06-init-prime.md",
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

fn examples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("exemplos")
}

#[test]
fn example_images_exist() {
    let dir = examples_dir();
    let page =
        std::fs::read_to_string(dir.join("diagramas-reais.md")).expect("lê página de exemplos");
    let mut images = 0_usize;
    let mut missing: Vec<String> = Vec::new();
    for fragment in page.split("](").skip(1) {
        let Some(end) = fragment.find(')') else {
            continue;
        };
        let Some(target) = fragment.get(..end) else {
            continue;
        };
        if !target.starts_with("img/") {
            continue;
        }
        images = images.saturating_add(1);
        if !dir.join(target).is_file() {
            missing.push(target.to_string());
        }
    }
    assert!(images > 0, "a página de exemplos não cita nenhuma imagem");
    assert!(
        missing.is_empty(),
        "faltam imagens em docs/exemplos/: {}",
        missing.join(", ")
    );
}
