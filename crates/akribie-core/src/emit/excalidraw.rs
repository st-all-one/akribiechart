//! Export `.excalidraw` derivado e determinístico (ADR-0004).
//!
//! Nada de relógio: `updated`/`created`/`versionNonce` são `0` e `seed` vem do id. `index` é
//! `a<base62(i):0>4V` — largura fixa (ordem lexicográfica = ordem numérica) e sufixo `V` para nunca
//! terminar em `0`, que o `validateOrderKey` do Excalidraw rejeita. Elementos saem na ordem
//! `group`, `edge`, `node`(+ texto do container), `note`.

use serde_json::{Value, json};

mod edges;
mod shapes;
mod support;

use self::edges::push_edge;
use self::shapes::{push_group, push_node, push_note};
use self::support::{index_key, set};

use crate::layout::Placed;
use crate::model::Chart;

/// Serializa o desenho em `.excalidraw` (JSON bonito, `\n` final).
pub fn excalidraw(chart: &Chart, placed: &Placed) -> String {
    let theme = chart.theme;
    let mut elements: Vec<Value> = Vec::new();
    for group in &placed.groups {
        push_group(&mut elements, group, theme, chart.font);
    }
    for edge in &placed.edges {
        push_edge(
            &mut elements,
            edge,
            placed,
            theme,
            chart.fontsize,
            chart.font,
        );
    }
    for node in &placed.nodes {
        push_node(&mut elements, node, theme, chart.fontsize, chart.font);
    }
    for note in &placed.notes {
        push_note(&mut elements, note, theme, chart.fontsize, chart.font);
    }
    for (position, element) in elements.iter_mut().enumerate() {
        set(element, "index", Value::from(index_key(position)));
    }
    let document = json!({
        "appState": { "gridSize": null, "viewBackgroundColor": theme.canvas },
        "elements": elements,
        "files": {},
        "source": "https://github.com/st-all-one/akribie",
        "type": "excalidraw",
        "version": 2,
    });
    let mut text = serde_json::to_string_pretty(&document).unwrap_or_default();
    text.push('\n');
    text
}

#[cfg(test)]
mod tests;
