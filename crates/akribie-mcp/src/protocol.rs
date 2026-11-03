//! JSON-RPC 2.0 sobre linhas: parse, despacho e resposta.
//!
//! Notificações (sem `id`) não recebem resposta. Erro de parse vira `-32700` com `id: null`,
//! método desconhecido `-32601`.

use serde_json::{Value, json};

use crate::tools;

/// Versão do protocolo MCP anunciada no `initialize`.
const PROTOCOL_VERSION: &str = "2024-11-05";

/// Trata uma linha; `None` quando não há o que responder.
pub(crate) fn handle(line: &str) -> Option<String> {
    let text = line.trim();
    if text.is_empty() {
        return None;
    }
    let request: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(error) => {
            return Some(render(&error_response(
                &Value::Null,
                -32700,
                &format!("JSON inválido: {error}"),
            )));
        }
    };
    let id = request.get("id").cloned()?;
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = request.get("params").cloned().unwrap_or(Value::Null);
    Some(render(&dispatch(method, &params, &id)))
}

fn dispatch(method: &str, params: &Value, id: &Value) -> Value {
    match method {
        "initialize" => ok(
            id,
            &json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "akribie-mcp",
                    "version": akribie_core::VERSION,
                },
            }),
        ),
        "ping" => ok(id, &json!({})),
        "tools/list" => ok(id, &tools::list()),
        "tools/call" => match tools::call(params) {
            Ok(result) => ok(id, &result),
            Err(message) => error_response(id, -32602, &message),
        },
        other => error_response(id, -32601, &format!("método não suportado: {other}")),
    }
}

fn ok(id: &Value, result: &Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error_response(id: &Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
}

fn render(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| {
        "{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32603,\"message\":\"falha ao serializar\"}}"
            .to_string()
    })
}
