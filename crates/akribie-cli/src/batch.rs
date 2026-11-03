//! Mecânica dos lotes JSONL, comum a `check --batch` e `render --batch` (§2.3).
//!
//! O núcleo separa as linhas (`pipeline::batch_lines`); aqui ficam as regras que valem nos dois
//! verbos: `name` obrigatório e restrito, e a falha de item com código estável para o envelope.
//! As chaves restantes continuam sendo validadas pela tabela de `--params` do próprio verbo.

use akribie_core::Error;
use akribie_core::pipeline::BatchLine;
use serde_json::{Map, Value};

/// Falha de um item: código estável (`invalid_input`, `not_found`, `io`, `schema`) + mensagem.
#[derive(Debug, Clone)]
pub(crate) struct ItemError {
    /// Código textual, o mesmo do envelope `--json`.
    pub(crate) code: &'static str,
    /// Mensagem auto-suficiente.
    pub(crate) message: String,
}

impl ItemError {
    /// Erro de uso do item (nome, origem, chave).
    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_input",
            message: message.into(),
        }
    }

    /// Erro de E/S traduzido da porta, com o código da `ErrorKind`.
    pub(crate) fn from_error(error: &Error) -> Self {
        Self {
            code: error.kind().code(),
            message: error.message().to_string(),
        }
    }
}

/// Item do lote com o nome validado e o objeto restante pronto para `params::parse`.
#[derive(Debug)]
pub(crate) struct Item {
    /// Posição no lote (0-based, sem contar linhas vazias).
    pub(crate) index: usize,
    /// Nome validado; base do arquivo de saída.
    pub(crate) name: String,
    /// Objeto sem `name`, serializado para o `params::parse` do verbo.
    pub(crate) params: String,
    /// Falha de JSONL ou de `name`; quando presente, o item não é executado.
    pub(crate) error: Option<ItemError>,
}

/// Converte as linhas do núcleo em itens, na ordem do arquivo.
pub(crate) fn items(lines: Vec<BatchLine>) -> Vec<Item> {
    lines.into_iter().map(item_of).collect()
}

fn item_of(line: BatchLine) -> Item {
    let fallback = format!("line-{}", line.number);
    if let Some(error) = line.error {
        return Item {
            index: line.index,
            name: fallback,
            params: String::new(),
            error: Some(ItemError::invalid(error)),
        };
    }
    let Some(Value::Object(mut map)) = line.value else {
        return Item {
            index: line.index,
            name: fallback,
            params: String::new(),
            error: Some(ItemError::invalid("o item precisa ser um objeto JSON")),
        };
    };
    match name_of(&mut map, &fallback) {
        Ok(name) => Item {
            index: line.index,
            name,
            params: Value::Object(map).to_string(),
            error: None,
        },
        Err(error) => Item {
            index: line.index,
            name: fallback,
            params: String::new(),
            error: Some(error),
        },
    }
}

/// `name` obrigatório, `[A-Za-z0-9_-]{1,64}`; removido do mapa antes da validação de chaves.
fn name_of(map: &mut Map<String, Value>, fallback: &str) -> Result<String, ItemError> {
    let Some(value) = map.remove("name") else {
        return Err(ItemError::invalid(format!(
            "item `{fallback}` sem `name` (obrigatório no lote)"
        )));
    };
    let Some(name) = value.as_str() else {
        return Err(ItemError::invalid(format!(
            "item `{fallback}`: `name` precisa ser string"
        )));
    };
    if !valid_name(name) {
        return Err(ItemError::invalid(format!(
            "item `{fallback}`: `name` deve casar [A-Za-z0-9_-]{{1,64}} (recebido `{name}`)"
        )));
    }
    Ok(name.to_string())
}

/// Nome de item: ASCII alfanumérico, `_` e `-`, de 1 a 64 caracteres.
fn valid_name(name: &str) -> bool {
    (1..=64).contains(&name.len())
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}
