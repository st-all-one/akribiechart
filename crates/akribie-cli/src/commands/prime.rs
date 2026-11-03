//! `akr prime`: gramática, regras e tabelas em texto — para colar em um agente.
//!
//! O prime é **contrato de bytes por versão** (golden em `tests/golden/`). O compacto é constante;
//! o `--long` anexa a BNF, as tabelas de chaves (derivadas de `syntax::option_specs`, a mesma fonte
//! do parser) e os tokens dos temas.

use std::fmt::Write as _;

use akribie_core::diag;
use akribie_core::syntax::{self, StmtKind};
use akribie_core::theme;

/// Prime compacto: o que um agente precisa saber para escrever `.akrc` correto na primeira tentativa.
pub(crate) const PRIME_COMPACT: &str = "\
akribie — `.akrc` → accessible SVG, raster, scene JSON, Excalidraw

Write topology, not coordinates: declare nodes and edges; `akr` computes geometry.
One statement per line, UTF-8 without BOM. Blank lines are free. `#` starts a comment.

chart v1 \"Title\" [layout=flow|tree|mindmap|grid|stack|free] [direction=lr|tb|bt|rl]
               [theme=paper|slate] [fontsize=20] [pad=32] [lang=en] [sketch=on|off]
               [font=hand|sans] [doc=\"path#anchor\"]
group ID [\"Label\"] [fill=…] [stroke=…] [pad=24] [in=ID,ID]
node ID [\"Label\"] [shape=rect|rounded|circle|diamond|cloud|cylinder|hexagon|note|text]
                 [fill=none|paper|blue|green|amber|red|violet]
                 [fillstyle=hachure|solid|cross-hatch|zigzag]
                 [stroke=ink|muted|none|paper|blue|green|amber|red|violet]
                 [style=solid|dashed|dotted] [width=2] [in=ID,ID] [at=x,y] [size=w,h]
edge FROM -> TO [\"Label\"] [curve=line|arc|elbow] [head=arrow|none] [tail=arrow|none] [style=…]
note \"Text\" [fill=…] [at=x,y] [size=w,h] [near=ID]

Layouts: flow = pipeline (direction), tree = hierarchy, mindmap = topics, grid = catalog,
         stack = one column, free = declared positions (every node/note needs `at=x,y`).

Finish: `sketch=on` (default) is hand-drawn; `sketch=off` is technical (exact lines, real arcs).
Font: `font=hand` (default) is Excalifont; `font=sans` is Liberation Sans. Labels never touch a
line, a border or another block: edge labels interrupt their own line with a 6 px gap.

Example 1 — retry with backoff:

chart v1 \"Retry\" layout=flow direction=lr
node api \"API\" fill=blue
node retry \"Retry x3\" fill=amber shape=cloud
node dlq \"DLQ\" fill=red shape=cylinder
edge api -> retry \"5xx\"
edge retry -> api \"backoff\" curve=arc style=dashed
edge retry -> dlq \"exhausted\" style=dashed

Example 2 — grouped pipeline:

chart v1 \"Pipeline\" layout=flow direction=lr
group ingest \"Ingest\" fill=paper
node src \"Source\" in=ingest
node queue \"Queue\" in=ingest shape=cylinder
node worker \"Worker\"
edge src -> queue
edge queue -> worker \"job\"

Fixing errors: `akr check` exits 8 and prints `file:line:col: E0xx name: message` plus a `hint:`.
Rewrite only the reported line and resend. Never rewrite the whole document, never invent keys:
the set is closed and `E002`/`E003` list the valid values.

Accessibility is not optional: always declare the chart title (it becomes the SVG `<title>`); the
SVG already carries `role=\"img\"`, `<title>`, `<desc>` and one `aria-label` per element, and the
themes guarantee contrast (4.5:1 text, 3:1 outline). Never encode meaning by colour alone.

Exit codes: 0 ok | 1 `fmt --check` differs or `diff` found changes | 2 usage | 3 not found |
5 I/O | 8 invalid source | 70 internal.
";

/// BNF da gramática (usada pelo `--long`).
const BNF: &str = "\
    chart      = \"chart\" WS version WS quoted { WS option } ;
    version    = \"v1\" ;
    group      = \"group\" WS ident [ WS quoted ] { WS option } ;
    node       = \"node\" WS ident [ WS quoted ] { WS option } ;
    edge       = \"edge\" WS ident WS \"->\" WS ident [ WS quoted ] { WS option } ;
    note       = \"note\" WS quoted { WS option } ;
    option     = key \"=\" value ;
    value      = quoted | number | ident | pair | list ;
    pair       = number \",\" number ;
    list       = ident { \",\" ident } ;
    quoted     = \"\\\"\" { char - \">\" - \"\\\"\" - \"\\\\\" | escape } \"\\\"\" ;
    ident      = letter { letter | digit | \"_\" | \"-\" } ;
    number     = [ \"-\" ] digit { digit } [ \".\" digit ] ;
    statement  = chart | group | node | edge | note ;
    document   = statement { newline statement } [ newline ] ;
";

/// Anexa `args` a `text`; escrever em `String` nunca falha.
fn append(text: &mut String, args: std::fmt::Arguments<'_>) {
    text.write_fmt(args).unwrap_or(());
}

/// Profundidade do prime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Length {
    /// Só o compacto (golden por versão).
    Compact,
    /// Compacto + BNF + tabelas + temas (`--long`).
    Long,
}

/// Texto do prime: compacto ou compacto + BNF + tabelas + temas.
pub(crate) fn render(length: Length) -> String {
    let mut text = String::from(PRIME_COMPACT);
    if matches!(length, Length::Long) {
        text.push_str("\n## BNF\n\n");
        text.push_str(BNF);
        text.push_str("\n## Keys per statement\n");
        for kind in STATEMENTS {
            write_statement(&mut text, kind);
        }
        text.push_str("\n## Theme tokens\n");
        for name in syntax::THEMES {
            write_theme(&mut text, name);
        }
        text.push_str("\n## Diagnostics\n");
        for code in &diag::ALL {
            append(
                &mut text,
                format_args!("{} {} ({})\n", code.code, code.name, code.severity.as_str()),
            );
        }
    }
    text
}

const STATEMENTS: [StmtKind; 5] = [
    StmtKind::Chart,
    StmtKind::Group,
    StmtKind::Node,
    StmtKind::Edge,
    StmtKind::Note,
];

fn write_theme(text: &mut String, name: &str) {
    let Some(theme) = theme::theme(name) else {
        return;
    };
    append(
        text,
        format_args!("{name}: canvas={} ink={}\n", theme.canvas, theme.ink),
    );
    for (token, hex) in theme.palette {
        append(text, format_args!("  {token}={hex}\n"));
    }
}

fn write_statement(text: &mut String, kind: StmtKind) {
    append(text, format_args!("\n{}:\n", kind.keyword()));
    for spec in syntax::option_specs(kind) {
        append(
            text,
            format_args!("  {:9} {:5}", spec.key, spec.kind.as_str()),
        );
        if let Some(default) = spec.default {
            append(text, format_args!(" default={default}"));
        }
        if !spec.allowed.is_empty() {
            append(text, format_args!(" values={}", spec.allowed.join("|")));
        }
        if let Some((min, max)) = spec.range {
            if max >= f32::MAX {
                append(text, format_args!(" range={min}.."));
            } else {
                append(text, format_args!(" range={min}..{max}"));
            }
        }
        text.push('\n');
    }
}
