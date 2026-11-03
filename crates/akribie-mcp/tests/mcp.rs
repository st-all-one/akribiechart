//! Contrato de borda do servidor MCP: JSON-RPC por linha, erro como dado, servidor vivo.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "teste de integração"
)]

use std::io::Write;
use std::process::{Command, Stdio};

fn call(lines: &[&str]) -> Vec<serde_json::Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_akribie-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mcp");
    let mut stdin = child.stdin.take().expect("stdin");
    for line in lines {
        writeln!(stdin, "{line}").expect("escreve linha");
    }
    drop(stdin);
    let output = child.wait_with_output().expect("aguarda mcp");
    assert!(output.status.success(), "exit do servidor");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("resposta JSON"))
        .collect()
}

const CHART: &str = "chart v1 \\\"x\\\"\\nnode a \\\"A\\\"\\nnode b \\\"B\\\"\\nedge a -> b\\n";

#[test]
fn tools_list_announces_the_three_tools() {
    let responses = call(&[r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#]);
    assert_eq!(responses.len(), 1);
    let tools = responses[0]["result"]["tools"].as_array().expect("tools");
    let names: Vec<&str> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    assert_eq!(names, ["chart_check", "chart_render", "chart_diff"]);
}

#[test]
fn invalid_source_is_is_error_and_the_server_stays_alive() {
    let responses = call(&[
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"chart_check","arguments":{"source":"chart v1 \"x\"\nnode a shape=nope\n"}}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"chart_check","arguments":{"source":"chart v1 \"x\"\nnode a \"A\"\nnode b \"B\"\nedge a -> b\n"}}}"#,
    ]);
    assert_eq!(responses.len(), 2, "o servidor respondeu às duas");
    assert_eq!(responses[0]["result"]["isError"], true);
    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("texto");
    assert!(text.contains("E003"), "{text}");
    assert!(!text.contains("hint:"), "só a primeira linha: {text}");
    assert_eq!(responses[1]["result"]["isError"], serde_json::Value::Null);
}

#[test]
fn malformed_json_is_a_protocol_error_without_killing_the_server() {
    let responses = call(&["não é json", r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#]);
    assert_eq!(responses[0]["error"]["code"], -32700);
    assert_eq!(responses[1]["result"], serde_json::json!({}));
}

#[test]
fn render_png_returns_an_image_content_block() {
    let request = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"chart_render","arguments":{{"source":"{CHART}","format":"png","scale":1}}}}}}"#
    );
    let responses = call(&[request.as_str()]);
    let content = &responses[0]["result"]["content"][0];
    assert_eq!(content["type"], "image");
    assert_eq!(content["mimeType"], "image/png");
    let data = content["data"].as_str().expect("base64");
    assert!(data.starts_with("iVBORw0KGgo"), "assinatura PNG em base64");
}

#[test]
fn diff_reports_the_label_change() {
    let old = "chart v1 \\\"x\\\"\\nnode a \\\"A\\\"\\n";
    let new = "chart v1 \\\"x\\\"\\nnode a \\\"A2\\\"\\n";
    let request = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"chart_diff","arguments":{{"old_source":"{old}","new_source":"{new}"}}}}}}"#
    );
    let responses = call(&[request.as_str()]);
    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("texto");
    assert_eq!(text, "~ node a label: A -> A2");
}
