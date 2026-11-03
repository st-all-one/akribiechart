//! Superfície de linha de comando (`akr`).
//!
//! `stdout` é dados; `stderr` é log e uso. Nenhum comando depende de cor de terminal ou de largura
//! de terminal, porque a saída de `akr` é consumida por pipes e por agentes.

use clap::{Args, Parser, Subcommand};

/// Compila `.akrc` em diagramas determinísticos e acessíveis.
#[derive(Debug, Parser)]
#[command(
    name = "akr",
    version,
    about = "Compila .akrc em diagramas acessíveis (SVG, raster, scene, Excalidraw)",
    long_about = None,
    disable_help_subcommand = true
)]
pub(crate) struct Cli {
    /// Subcomando a executar.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// Subcomandos disponíveis.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Valida um `.akrc` e reporta diagnósticos posicionados.
    Check(CheckArgs),
    /// Normaliza um `.akrc` para a forma canônica.
    Fmt(FmtArgs),
    /// Publica a gramática, os tokens de tema e os códigos de diagnóstico em JSON.
    Schema,
    /// Compila o `.akrc` em SVG, `scene` JSON (stdout) ou raster (`--out`).
    Render(RenderArgs),
    /// Compara dois `.akrc` e reporta mudanças semânticas.
    Diff(DiffArgs),
    /// Converte um `.excalidraw` em `.akrc`.
    Import(ImportArgs),
    /// Instala `akribie.toml`, bloco de instruções em `AGENTS.md` e skill do agente.
    Init(InitArgs),
    /// Imprime a gramática e as regras, prontas para colar em um agente.
    Prompt(PromptArgs),
}

/// Argumentos de `akr init`.
#[derive(Debug, Args)]
pub(crate) struct InitArgs {
    /// Reescreve `akribie.toml` mesmo quando já existe.
    #[arg(long)]
    pub(crate) force: bool,
}

/// Argumentos de `akr prompt`.
#[derive(Debug, Args)]
pub(crate) struct PromptArgs {
    /// Anexa BNF completa, tabelas de chaves e tokens de tema.
    #[arg(long)]
    pub(crate) long: bool,
}

/// Argumentos de `akr check`.
#[derive(Debug, Args)]
pub(crate) struct CheckArgs {
    /// Arquivo `.akrc`; `-` lê da entrada padrão.
    pub(crate) file: Option<String>,

    /// Parâmetros em JSON; substitui posicionais e flags do verbo.
    #[arg(long, value_name = "JSON")]
    pub(crate) params: Option<String>,

    /// Emite o envelope JSON em vez de texto.
    #[arg(long)]
    pub(crate) json: bool,

    /// Promove warning a erro (exit 8).
    #[arg(long)]
    pub(crate) strict: bool,

    /// Resolve a linhagem `doc=` no disco (W004).
    #[arg(long)]
    pub(crate) docs: bool,

    /// Lote JSONL: um objeto por linha (`{"name","path"|"source"}`).
    #[arg(long, value_name = "FILE")]
    pub(crate) batch: Option<String>,
}

/// Argumentos de `akr fmt`.
#[derive(Debug, Args)]
pub(crate) struct FmtArgs {
    /// Arquivo `.akrc`; `-` lê da entrada padrão.
    pub(crate) file: Option<String>,

    /// Parâmetros em JSON; substitui posicionais e flags do verbo.
    #[arg(long, value_name = "JSON")]
    pub(crate) params: Option<String>,

    /// Grava o resultado no próprio arquivo.
    #[arg(long)]
    pub(crate) write: bool,

    /// Sai 1 quando o arquivo não estiver na forma canônica.
    #[arg(long)]
    pub(crate) check: bool,
}

/// Argumentos de `akr render`.
#[derive(Debug, Args)]
pub(crate) struct RenderArgs {
    /// Arquivo `.akrc`; `-` lê da entrada padrão.
    pub(crate) file: Option<String>,

    /// Parâmetros em JSON; substitui posicionais e flags do verbo.
    #[arg(long, value_name = "JSON")]
    pub(crate) params: Option<String>,

    /// `svg` (padrão), `scene`, `excalidraw`, `png`, `jpg` ou `avif`.
    #[arg(long, value_name = "FORMAT")]
    pub(crate) format: Option<String>,

    /// Arquivo de saída (obrigatório para raster).
    #[arg(long, value_name = "PATH")]
    pub(crate) out: Option<String>,

    /// Fator de escala da rasterização (padrão 2).
    #[arg(long, value_name = "N")]
    pub(crate) scale: Option<f32>,

    /// Emite o envelope JSON em vez de dados.
    #[arg(long)]
    pub(crate) json: bool,

    /// Diretório de saída do lote (`--batch`).
    #[arg(long, value_name = "DIR")]
    pub(crate) out_dir: Option<String>,

    /// Lote JSONL: um objeto por linha (`{"name","path"|"source"}`).
    #[arg(long, value_name = "FILE")]
    pub(crate) batch: Option<String>,

    /// Lê o fonte na revisão `REV` do Git (`git show REV:FILE`).
    #[arg(long, value_name = "REV")]
    pub(crate) at: Option<String>,

    /// Valida e reporta a saída sem gravar arquivo.
    #[arg(long)]
    pub(crate) dry_run: bool,
}

/// Argumentos de `akr diff`.
#[derive(Debug, Args)]
pub(crate) struct DiffArgs {
    /// Arquivo antigo (`.akrc` ou `-`).
    pub(crate) old: Option<String>,

    /// Arquivo novo (`.akrc` ou `-`).
    pub(crate) new: Option<String>,

    /// Parâmetros em JSON; substitui posicionais e flags do verbo.
    #[arg(long, value_name = "JSON")]
    pub(crate) params: Option<String>,

    /// Emite o envelope JSON em vez de linhas.
    #[arg(long)]
    pub(crate) json: bool,

    /// Compara o arquivo atual com a revisão `REV` do Git.
    #[arg(long, value_name = "REV")]
    pub(crate) at: Option<String>,
}

/// Argumentos de `akr import`.
#[derive(Debug, Args)]
pub(crate) struct ImportArgs {
    /// Arquivo `.excalidraw`; `-` lê da entrada padrão.
    pub(crate) file: Option<String>,

    /// Parâmetros em JSON; substitui posicionais e flags do verbo.
    #[arg(long, value_name = "JSON")]
    pub(crate) params: Option<String>,

    /// Emite o envelope JSON em vez da fonte.
    #[arg(long)]
    pub(crate) json: bool,
}
