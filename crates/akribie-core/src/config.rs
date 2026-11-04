//! Configuração do projeto: `akribie.toml`.
//!
//! O arquivo é um **subconjunto determinístico de TOML** — `chave = valor`, uma por linha,
//! comentários de linha inteira com `#`. Nada de tabelas, arrays, datas ou interpolação: o que o
//! `akr init` escreve é o que o `akr` lê, e um erro de digitação vira mensagem, nunca silêncio.
//!
//! Os valores passam pelo **mesmo** léxico e pela **mesma** validação do `.akrc`
//! ([`crate::syntax::validate`]); a configuração não é um segundo dialeto.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::lex;
use crate::model::{Direction, FontFamily, Layout};
use crate::ports::Fs;
use crate::span::Span;
use crate::syntax::{self, StmtKind};
use crate::theme::{PAPER, Theme, theme};

/// Nome do arquivo de configuração.
pub const FILE_NAME: &str = "akribie.toml";

/// Chaves aceitas no arquivo, na ordem em que `init` as escreve.
pub const KEYS: [&str; 8] = [
    "layout",
    "direction",
    "theme",
    "fontsize",
    "pad",
    "lang",
    "sketch",
    "font",
];

/// Parâmetros do projeto. Ausente o arquivo, vale [`Self::default`].
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Perfil de layout padrão.
    pub layout: Layout,
    /// Direção padrão do fluxo.
    pub direction: Direction,
    /// Tema padrão.
    pub theme: &'static Theme,
    /// Tamanho de fonte padrão (14–96).
    pub fontsize: f32,
    /// Margem externa padrão (0–256).
    pub pad: f32,
    /// Idioma padrão (BCP-47 simples).
    pub lang: String,
    /// Traço à mão (`true`) ou traço técnico (`false`).
    pub sketch: bool,
    /// Família tipográfica padrão.
    pub font: FontFamily,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layout: Layout::Flow,
            direction: Direction::Lr,
            theme: theme("paper").unwrap_or(&PAPER),
            fontsize: 20.0,
            pad: 32.0,
            lang: "pt-BR".to_string(),
            sketch: false,
            font: FontFamily::Sans,
        }
    }
}

impl Config {
    /// Lê o arquivo de configuração.
    ///
    /// # Errors
    /// `InvalidInput` para linha malformada, chave desconhecida ou valor fora do contrato.
    pub fn from_toml(source: &str) -> Result<Self> {
        let mut config = Self::default();
        for (index, raw) in source.lines().enumerate() {
            let line_number = u32::try_from(index.saturating_add(1)).unwrap_or(u32::MAX);
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, raw_value)) = line.split_once('=') else {
                return Err(Error::invalid_input(format!(
                    "{FILE_NAME}:{line_number}: esperado `chave = valor`, encontrado `{line}`"
                )));
            };
            config.apply(key.trim(), raw_value.trim(), line_number)?;
        }
        Ok(config)
    }

    fn apply(&mut self, key: &str, raw: &str, line: u32) -> Result<()> {
        let spec = syntax::spec_for(StmtKind::Chart, key).filter(|_spec| KEYS.contains(&key));
        let Some(spec) = spec else {
            return Err(Error::invalid_input(format!(
                "{FILE_NAME}:{line}: chave desconhecida `{key}` (aceitas: {})",
                KEYS.join(", ")
            )));
        };
        let raw = if spec.kind == syntax::ValKind::Bareword {
            raw.strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
                .unwrap_or(raw)
        } else {
            raw
        };
        let tokens = lex::lex(raw, Span::line_start(0)).map_err(|diagnostic| {
            Error::invalid_input(format!("{FILE_NAME}:{line}: {}", diagnostic.message))
        })?;
        let Some(token) = tokens.first() else {
            return Err(Error::invalid_input(format!(
                "{FILE_NAME}:{line}: valor ausente para `{key}`"
            )));
        };
        let value = syntax::validate(spec, token).map_err(|error| {
            Error::invalid_input(format!("{FILE_NAME}:{line}: `{key}`: {}", error.message))
        })?;
        match key {
            "layout" => {
                self.layout =
                    Layout::from_name(value.as_str().unwrap_or("flow")).unwrap_or(Layout::Flow);
            }
            "direction" => {
                self.direction =
                    Direction::from_name(value.as_str().unwrap_or("lr")).unwrap_or(Direction::Lr);
            }
            "theme" => {
                self.theme = theme(value.as_str().unwrap_or("paper")).unwrap_or(&PAPER);
            }
            "fontsize" => self.fontsize = value.as_num().unwrap_or(self.fontsize),
            "pad" => self.pad = value.as_num().unwrap_or(self.pad),
            "lang" => self.lang = value.as_str().unwrap_or(&self.lang).to_string(),
            "sketch" => {
                self.sketch = value.as_str().map_or(self.sketch, |name| name == "on");
            }
            "font" => {
                self.font = value
                    .as_str()
                    .and_then(FontFamily::from_name)
                    .unwrap_or(self.font);
            }
            _ => {}
        }
        Ok(())
    }

    /// Procura `akribie.toml` subindo do diretório de `base` até a raiz; o primeiro vence.
    ///
    /// `base` pode ser o arquivo `.akrc` (usa o diretório) ou um diretório. Sem `base`, devolve os
    /// defaults — `stdin` não tem projeto.
    ///
    /// # Errors
    /// Propaga falhas de leitura do arquivo encontrado.
    pub fn discover(fs: &dyn Fs, base: Option<&Path>) -> Result<Self> {
        let Some(base) = base else {
            return Ok(Self::default());
        };
        let mut dir: PathBuf = if base.is_dir() || base.extension().is_none() {
            base.to_path_buf()
        } else {
            base.parent()
                .map_or_else(|| base.to_path_buf(), Path::to_path_buf)
        };
        loop {
            let candidate = dir.join(FILE_NAME);
            if fs.exists(&candidate) {
                return Self::from_toml(&fs.read(&candidate)?);
            }
            let Some(parent) = dir.parent() else {
                return Ok(Self::default());
            };
            dir = parent.to_path_buf();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::fakes::MemFs;
    use std::path::PathBuf;

    fn mem_files(entries: &[(&str, &str)]) -> MemFs {
        entries.iter().fold(MemFs::new(), |fs, (path, text)| {
            fs.with_file(PathBuf::from(path), (*text).to_string())
        })
    }

    #[test]
    fn defaults_without_file() {
        let config = Config::default();
        assert_eq!(config.layout, Layout::Flow);
        assert_eq!(config.direction, Direction::Lr);
        assert_eq!(config.theme.name, "paper");
        assert_eq!(config.fontsize, 20.0);
        assert_eq!(config.pad, 32.0);
        assert_eq!(config.lang, "pt-BR");
        assert!(!config.sketch);
        assert_eq!(config.font, FontFamily::Sans);
    }

    #[test]
    fn reads_finish_and_font_keys() {
        let config = Config::from_toml("sketch = off\nfont = sans\n").expect("config válido");
        assert!(!config.sketch);
        assert_eq!(config.font, FontFamily::Sans);
        let config = Config::from_toml("sketch = on\nfont = hand\n").expect("config válido");
        assert!(config.sketch);
        assert_eq!(config.font, FontFamily::Hand);
        assert!(Config::from_toml("sketch = maybe\n").is_err());
        assert!(Config::from_toml("font = serif\n").is_err());
    }

    #[test]
    fn reads_every_key() {
        let config = Config::from_toml(
            "# tipo de diagrama\nlayout = tree\ndirection = tb\ntheme = slate\nfontsize = 24\npad = 48\nlang = \"pt-BR\"\n",
        );
        let config = config.expect("config válido");
        assert_eq!(config.layout, Layout::Tree);
        assert_eq!(config.direction, Direction::Tb);
        assert_eq!(config.theme.name, "slate");
        assert_eq!(config.fontsize, 24.0);
        assert_eq!(config.pad, 48.0);
        assert_eq!(config.lang, "pt-BR");
    }

    #[test]
    fn unknown_key_is_reported_with_line() {
        let error = Config::from_toml("layout = flow\ncor = \"azul\"\n").expect_err("deve falhar");
        let message = error.message();
        assert!(message.contains("akribie.toml:2"), "{message}");
        assert!(message.contains("cor"), "{message}");
    }

    #[test]
    fn invalid_value_uses_akrc_rules() {
        let error = Config::from_toml("layout = spiral\n").expect_err("deve falhar");
        assert!(error.message().contains("layout"), "{}", error.message());
        let error = Config::from_toml("fontsize = 400\n").expect_err("deve falhar");
        assert!(error.message().contains("fontsize"), "{}", error.message());
    }

    #[test]
    fn discover_walks_up_and_first_wins() {
        let fs = mem_files(&[
            ("/repo/akribie.toml", "layout = grid\n"),
            ("/repo/docs/akribie.toml", "layout = stack\n"),
        ]);
        let config = Config::discover(&fs, Some(Path::new("/repo/docs/diagram.akrc"))).expect("ok");
        assert_eq!(config.layout, Layout::Stack);
        let config = Config::discover(&fs, Some(Path::new("/repo/src/a/b.akrc"))).expect("ok");
        assert_eq!(config.layout, Layout::Grid);
    }

    #[test]
    fn discover_without_file_returns_defaults() {
        let fs = mem_files(&[]);
        let config = Config::discover(&fs, Some(Path::new("/nada/diagram.akrc"))).expect("ok");
        assert_eq!(config, Config::default());
        let config = Config::discover(&fs, None).expect("ok");
        assert_eq!(config, Config::default());
    }
}
