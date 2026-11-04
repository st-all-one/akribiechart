use akribie_core::ports::fakes::MemFs;

use super::{AGENTS, CONFIG, Force, Status, execute};

#[test]
fn first_run_creates_config_block_and_skill() {
    let fs = MemFs::new();
    let outcome = execute(Force::No, &fs).expect("init");
    assert_eq!(
        outcome.entries,
        vec![
            (CONFIG.to_string(), Status::Created),
            (AGENTS.to_string(), Status::Created),
            (".agents/skills/akr/SKILL.md".to_string(), Status::Created),
        ]
    );
    let agents = fs.contents(AGENTS).expect("AGENTS.md");
    assert!(agents.contains("<!-- akribie:start -->"));
    assert!(agents.contains("<!-- akribie:end -->"));
    let config = fs.contents(CONFIG).expect("config");
    assert!(config.contains("layout = flow"));
    assert!(config.contains("sketch = off"));
    assert!(config.contains("font = sans"));
    assert!(config.contains("lang = \"pt-BR\""));
}

#[test]
fn second_run_is_idempotent_and_does_not_duplicate_the_block() {
    let fs = MemFs::new();
    execute(Force::No, &fs).expect("init");
    let before = fs.contents(AGENTS).expect("AGENTS.md");
    let outcome = execute(Force::No, &fs).expect("init repetido");
    assert!(
        outcome
            .entries
            .iter()
            .all(|(_, status)| *status == Status::Unchanged)
    );
    assert_eq!(fs.contents(AGENTS).expect("AGENTS.md"), before);
    assert_eq!(
        before.matches("<!-- akribie:start -->").count(),
        1,
        "{before}"
    );
}

#[test]
fn existing_block_is_replaced_in_place_and_keeps_the_rest() {
    let fs = MemFs::new().with_file(
        AGENTS,
        "# Projeto\n\n<!-- akribie:start -->\n## velho\n<!-- akribie:end -->\n\nfim\n",
    );
    let outcome = execute(Force::No, &fs).expect("init");
    assert_eq!(outcome.entries[1].1, Status::Updated);
    let agents = fs.contents(AGENTS).expect("AGENTS.md");
    assert!(
        agents.starts_with("# Projeto\n\n<!-- akribie:start -->"),
        "{agents}"
    );
    assert!(agents.ends_with("fim\n"), "{agents}");
    assert!(!agents.contains("## velho"), "{agents}");
}

#[test]
fn foreign_skill_is_preserved() {
    let fs = MemFs::new().with_file(".agents/skills/akr/SKILL.md", "# skill de terceiro\n");
    let outcome = execute(Force::No, &fs).expect("init");
    assert_eq!(outcome.entries[2].1, Status::Foreign);
    assert_eq!(
        fs.contents(".agents/skills/akr/SKILL.md").expect("skill"),
        "# skill de terceiro\n"
    );
}

#[test]
fn force_rewrites_the_config() {
    let fs = MemFs::new().with_file(CONFIG, "layout = grid\n");
    let outcome = execute(Force::Yes, &fs).expect("init");
    assert_eq!(outcome.entries[0].1, Status::Updated);
    assert!(
        fs.contents(CONFIG)
            .expect("config")
            .contains("layout = flow")
    );
}

#[test]
fn installed_skill_is_the_repository_skill() {
    let fs = MemFs::new();
    execute(Force::No, &fs).expect("init");
    assert_eq!(
        fs.contents(".agents/skills/akr/SKILL.md").expect("skill"),
        include_str!("../../../../../SKILL.md")
    );
}

#[test]
fn agents_block_covers_the_whole_workflow() {
    let fs = MemFs::new();
    execute(Force::No, &fs).expect("init");
    let agents = fs.contents(AGENTS).expect("AGENTS.md");
    for needle in [
        "akr check",
        "akr fmt",
        "akr render",
        "akr diff",
        "akr import",
        "akr schema",
        "akribie.toml",
        "akribie-mcp",
    ] {
        assert!(agents.contains(needle), "bloco sem {needle}:\n{agents}");
    }
}
