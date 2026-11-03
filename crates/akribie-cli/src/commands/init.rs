//! `akr init`: configuração do projeto, bloco de instruções e skill do agente.
//!
//! Idempotente por construção: `akribie.toml` só é reescrito com `--force`, o bloco em `AGENTS.md`
//! vive entre marcadores de linha inteira, e a skill só é tocada quando carrega o marcador de
//! versão (arquivo de terceiro no mesmo caminho é preservado).

use std::path::Path;

use akribie_core::config::FILE_NAME;
use akribie_core::error::Result;
use akribie_core::ports::Fs;

/// Nome do arquivo de configuração (mesma constante que o núcleo lê).
const CONFIG: &str = FILE_NAME;
/// Arquivo de instruções para agentes, na raiz do projeto.
const AGENTS: &str = "AGENTS.md";
/// Skill instalada por `init` (um diretório por skill).
const SKILL: &str = ".agents/skills/akr/SKILL.md";

/// Marcador de início do bloco em `AGENTS.md`.
const BLOCK_START: &str = "<!-- akribie:start -->";
/// Marcador de fim do bloco em `AGENTS.md`.
const BLOCK_END: &str = "<!-- akribie:end -->";
/// Marca de versão da skill (o que autoriza `init` a reescrevê-la).
const SKILL_MARK: &str = "<!-- akribie:skill:version: 1 -->";

/// Autorização para reescrever a configuração existente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Force {
    /// Conserva `akribie.toml` quando já existe.
    No,
    /// Reescreve mesmo existente (`--force`).
    Yes,
}

/// O que aconteceu com um arquivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    /// Não existia; foi criado.
    Created,
    /// Existia e foi atualizado.
    Updated,
    /// Já estava como `init` escreveria.
    Unchanged,
    /// Existia sem marcador reconhecível; foi preservado.
    Foreign,
}

/// Anexa `args` a `text`; escrever em `String` nunca falha.
fn append(text: &mut String, args: std::fmt::Arguments<'_>) {
    use std::fmt::Write as _;
    text.write_fmt(args).unwrap_or(());
}

impl Status {
    /// Rótulo em português usado na saída de texto.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Created => "novo",
            Self::Updated => "atualizado",
            Self::Unchanged => "inalterado",
            Self::Foreign => "preservado",
        }
    }
}

/// Resultado de `akr init`, na ordem em que os arquivos foram tratados.
#[derive(Debug)]
pub(crate) struct InitOutcome {
    /// `(caminho, status)` de cada arquivo.
    pub(crate) entries: Vec<(String, Status)>,
}

impl InitOutcome {
    /// Uma linha por arquivo, na ordem de escrita.
    pub(crate) fn render(&self) -> String {
        let mut text = String::new();
        for (path, status) in &self.entries {
            append(&mut text, format_args!("{}: {path}\n", status.label()));
        }
        text.push_str(
            "próximos: `akr check chart.akrc`, `akr fmt chart.akrc --write` e `akr prime` (gramática)\n",
        );
        text
    }
}

/// Executa a instalação.
///
/// # Errors
/// `Io` quando algum arquivo não puder ser lido ou escrito.
pub(crate) fn execute(force: Force, fs: &dyn Fs) -> Result<InitOutcome> {
    let config = write_config(fs, force)?;
    let agents = update_agents(fs)?;
    let skill = write_skill(fs)?;
    Ok(InitOutcome {
        entries: vec![
            (CONFIG.to_string(), config),
            (AGENTS.to_string(), agents),
            (SKILL.to_string(), skill),
        ],
    })
}

fn write_config(fs: &dyn Fs, force: Force) -> Result<Status> {
    let path = Path::new(CONFIG);
    if fs.exists(path) && matches!(force, Force::No) {
        return Ok(Status::Unchanged);
    }
    let existed = fs.exists(path);
    fs.write(path, config_text().as_bytes())?;
    Ok(if existed {
        Status::Updated
    } else {
        Status::Created
    })
}

fn update_agents(fs: &dyn Fs) -> Result<Status> {
    let path = Path::new(AGENTS);
    if !fs.exists(path) {
        fs.write(path, full_block().as_bytes())?;
        return Ok(Status::Created);
    }
    let current = fs.read(path)?;
    let next = match (current.find(BLOCK_START), current.find(BLOCK_END)) {
        (Some(start), Some(end)) if end > start => {
            let head = &current[..start];
            let tail = &current[end + BLOCK_END.len()..];
            format!("{head}{BLOCK_START}\n{}{BLOCK_END}{tail}", block_between())
        }
        _ => {
            let mut next = current.clone();
            if !next.ends_with('\n') {
                next.push('\n');
            }
            next.push('\n');
            next.push_str(&full_block());
            next
        }
    };
    if next == current {
        return Ok(Status::Unchanged);
    }
    fs.write(path, next.as_bytes())?;
    Ok(Status::Updated)
}

/// Bloco completo, marcadores inclusive, com `\n` final.
fn full_block() -> String {
    format!("{BLOCK_START}\n{}{BLOCK_END}\n", block_between())
}

fn write_skill(fs: &dyn Fs) -> Result<Status> {
    let path = Path::new(SKILL);
    if fs.exists(path) {
        let current = fs.read(path)?;
        if !current.contains(SKILL_MARK) {
            return Ok(Status::Foreign);
        }
        if current == skill_text() {
            return Ok(Status::Unchanged);
        }
        fs.write(path, skill_text().as_bytes())?;
        return Ok(Status::Updated);
    }
    fs.write(path, skill_text().as_bytes())?;
    Ok(Status::Created)
}

fn config_text() -> String {
    let mut text = String::from(
        "# akribie.toml — parâmetros do projeto (gerado por `akr init`).\n\
         # Chaves: layout, direction, theme, fontsize, pad, lang, sketch, font.\n\
         # Valores válidos e defaults: `akr schema`.\n\
         # Precedência: opção no `.akrc` > este arquivo > default do binário.\n\
         # sketch=off + font=sans: traço técnico (linhas exatas) e Liberation Sans.\n",
    );
    // Valores de enum/token/número saem sem aspas (o léxico do `.akrc` é a fonte); só texto puro
    // (`lang`) é string.
    text.push_str("layout = flow\n");
    text.push_str("direction = lr\n");
    text.push_str("theme = paper\n");
    text.push_str("fontsize = 20\n");
    text.push_str("pad = 32\n");
    text.push_str("lang = \"en\"\n");
    text.push_str("sketch = on\n");
    text.push_str("font = hand\n");
    text
}

/// Corpo do bloco (sem os marcadores): o protocolo de uso completo, em formato de AGENTS.md.
fn block_between() -> String {
    String::from(
        "## akribie\n\
         \n\
         Diagramas são `.akrc` versionados; SVG/PNG/scene/excalidraw são derivados — nunca edite o derivado.\n\
         \n\
         Fluxo: `akr check chart.akrc` → `akr fmt chart.akrc --write` → `akr render chart.akrc > chart.svg`.\n\
         - `akr check` sai 8 em erro; corrija **só** a linha `arquivo:linha:col: E0xx … hint:` e reenvie\n\
         \x20 (`--strict` promove aviso a erro; `--docs` resolve `doc=`).\n\
         - `akr fmt --check` sai 1 quando a forma canônica difere; `--write` grava.\n\
         - `akr render --format png|jpg|avif --out chart.png [--scale 2]` (raster exige `--out`).\n\
         - `akr render --format scene|excalidraw` e `--dry-run` depuram sem escrever.\n\
         - `akr diff OLD NEW [--json]` mostra `+`/`-`/`~` por id; exit 1 quando há mudança.\n\
         - `akr check --batch lote.jsonl` e `akr render --batch lote.jsonl --out-dir dist/` processam N charts;\n\
         \x20 `--params '<json>'` substitui integralmente as flags do verbo.\n\
         - `akr import desenho.excalidraw > desenho.akrc`; `akr schema` lista chaves e valores fechados;\n\
         \x20 `akr prime [--long]` é o guia de gramática (BNF, tabelas e temas no `--long`).\n\
         \n\
         Configuração: `akribie.toml` (`layout`, `direction`, `theme`, `fontsize`, `pad`, `lang`, `sketch`,\n\
         `font`); precedência: opção no `.akrc` > arquivo > default. `sketch=off font=sans` = traço técnico.\n\
         \n\
         Acessibilidade é requisito: título sempre presente (`<title>`), `<desc>` com a topologia,\n\
         `aria-label` por elemento, contraste garantido pelos tokens; nunca codifique sentido só por cor.\n\
         \n\
         MCP: `akribie-mcp` expõe `chart_check`, `chart_render` e `chart_diff` (JSON-RPC por stdio).\n\
         ",
    )
}

fn skill_text() -> String {
    // A skill instalada é o **mesmo** documento versionado na raiz; `include_str!` garante que
    // `akr init` nunca instale um guia defasado em relação ao `SKILL.md` entregue.
    include_str!("../../../../SKILL.md").to_string()
}

#[cfg(test)]
mod tests;
