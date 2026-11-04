# Matriz de aceitação

Definição de pronto, linha a linha. Cada linha amarra um comportamento observável a um teste
existente; linha sem teste é linha em aberto (fase indicada).

Colunas:

- **pipe** — comportamento em pipe (`stdout` = dados, `stderr` = logs, EPIPE = exit 0).
- **`--json`** — forma do envelope/`data` correspondente.
- **erro+exit** — código de diagnóstico e exit code em caso de falha.
- **estado/arquivo** — o que é escrito (e onde) ou o que permanece inalterado.
- **teste** — arquivo e nome do teste que cobre a linha.

A tabela é preenchida pelas fases M1–M8, conforme `docs/plan/01-fases.md`. Nenhuma linha pode ficar
com `pendente` ao fim de M8.

## `akr init`

| # | caso | pipe | `--json` | erro+exit | estado/arquivo | teste |
|---|---|---|---|---|---|---|
| 1 | cria config, bloco e skill | relatório `novo:`/`atualizado:`/`inalterado:`/`preservado:` + `próximos:` | — | exit 0 | `akribie.toml`, bloco em `AGENTS.md`, `.agents/skills/akr/SKILL.md` | `akribie-cli/tests/init.rs::init_installs_files_and_the_second_run_changes_nothing`, `::init_config_is_read_by_check`; `akribie-cli/src/commands/init/tests.rs::first_run_creates_config_block_and_skill` |
| 2 | segunda execução é idempotente | três linhas `inalterado:` | — | exit 0 | bloco não duplica; skill e config intactos; arquivo de terceiro é `preservado` | `akribie-cli/src/commands/init/tests.rs::second_run_is_idempotent_and_does_not_duplicate_the_block`, `::existing_block_is_replaced_in_place_and_keeps_the_rest`, `::foreign_skill_is_preserved`, `::force_rewrites_the_config` |

## `akr prime`

| # | caso | pipe | `--json` | erro+exit | estado/arquivo | teste |
|---|---|---|---|---|---|---|
| 1 | compacto é byte-idêntico | dois `akr prime` = mesmos bytes; golden confere | — | — | — | `akribie-cli/tests/prime.rs::compact_prime_is_byte_identical_and_matches_the_golden`; golden `tests/golden/prime-compact.txt` |
| 2 | `--long` anexa BNF e tabelas | começa pelo compacto + BNF + chaves + temas + códigos | — | — | — | `akribie-cli/tests/prime.rs::long_appends_bnf_tables_and_theme_tokens`; golden `tests/golden/prime-long.txt` |

## `akr check`

| # | caso | pipe | `--json` | erro+exit | estado/arquivo | teste |
|---|---|---|---|---|---|---|
| 1 | fonte válida (arquivo, stdin, `-`) | silencioso, exit 0 | `success:true`, `data.diagnostics:[]` | — | nada escrito | `akribie-cli/tests/check.rs::valid_source_via_stdin_is_silent_and_succeeds`, `::file_input_works`, `::dash_reads_stdin`, `::config_from_project_directory_changes_layout` |
| 2 | fonte inválida (≥1 `E`) | `arquivo:linha:col: CODE nome: msg` + `hint:` | `success:false`, `error.code:"schema"`, `data.diagnostics[]` | E00x + exit 8 | nada escrito | `akribie-cli/tests/check.rs::invalid_value_exits_8_with_actionable_hint`, `::json_envelope_carries_diagnostics`; `akribie-core/tests/parse.rs`; `akribie-core/src/resolve/tests.rs` (E004/E005/E006) |
| 3 | `--strict` promove warning | mesmos diagnósticos, exit 8 | `success:false`, `error.code:"schema"` | exit 8 | nada escrito | `akribie-cli/tests/check.rs::strict_promotes_warnings_to_failure` |
| 4 | `--batch` sequencial | `<name>  ok  <n> aviso(s)` / `<name>  erro  <CODE> <msg>`, na ordem | envelope `{ok,failed,items[]}`, cada item com `diagnostics`/`error` | exit 0 se ao menos um item passa; 8 se todos; 2 com lote vazio ou `--params`/`FILE` | nada escrito; `docs`/`strict` valem por item | `akribie-cli/tests/check.rs::batch_reports_each_item_and_exits_zero_when_partially_valid`, `::batch_fails_only_when_every_item_fails`, `::batch_item_needs_valid_name_and_single_source`, `::batch_strict_is_per_item`, `::batch_json_has_counts_and_items`; `akribie-core/src/pipeline.rs` (`batch_lines`) |
| 5 | `--docs` resolve linhagem | `W004` quando arquivo ou âncora não existem | idem, em `data.diagnostics` | warning não bloqueia (exit 0) | lê `doc=` relativo ao `.akrc`; âncora = texto sem caixa ou slug do título | `akribie-cli/tests/check.rs::docs_lineage_resolves_relative_to_the_source`, `::docs_lineage_reports_missing_document_or_anchor` |

## `akr fmt`

| # | caso | pipe | `--json` | erro+exit | estado/arquivo | teste |
|---|---|---|---|---|---|---|
| 1 | normaliza para forma canônica | forma canônica em `stdout` | — | exit 0 | inalterado | `akribie-cli/tests/fmt.rs::unordered_source_becomes_canonical`, `::canonical_form_is_idempotent` |
| 2 | `--write` grava | stdout vazio | — | exit 0 | arquivo reescrito no lugar | `akribie-cli/tests/fmt.rs::write_rewrites_the_file_in_place` |
| 3 | `--check` detecta diferença | stdout vazio (exit 0) ou nada | — | exit 1 quando difere | inalterado | `akribie-cli/tests/fmt.rs::check_accepts_canonical_and_rejects_divergence` |
| 4 | `--write` sem arquivo | — | — | exit 2 | nada escrito | `akribie-cli/tests/fmt.rs::write_without_a_file_is_usage_error` |
| 5 | `--write` com `--check` | — | — | exit 2 | nada escrito | `akribie-cli/tests/fmt.rs::write_and_check_are_exclusive` |
| 6 | `--params` substitui posicionais e flags | forma canônica em `stdout` | — | exit 0; posicional junto é exit 2 | inalterado | `akribie-cli/tests/fmt.rs::params_source_replaces_flags`, `::params_plus_positional_is_usage_error` |
| 7 | fonte inválida | diagnósticos em `stdout` | — | exit 8 | nada escrito | `akribie-cli/tests/fmt.rs::invalid_source_reports_diagnostics_and_exits_8` |
| 8 | documento vazio e comentário sem statement | vazio / comentários ao fim | — | exit 0 | — | `akribie-cli/tests/fmt.rs::empty_document_stays_empty`, `::comments_without_statement_stay_at_the_end` |

## `akr render`

| # | caso | pipe | `--json` | erro+exit | estado/arquivo | teste |
|---|---|---|---|---|---|---|
| 1 | SVG em stdout | `<svg>` válido em `xmllint`, `role`/`title`/`desc` | — | exit 0 | nada escrito | `akribie-cli/tests/render.rs::svg_goes_to_stdout_with_accessible_marks`, `::svg_with_out_writes_the_file_and_stays_silent` |
| 2 | determinismo (dois renders) | bytes idênticos | — | — | — | `akribie-cli/tests/render.rs::two_renders_are_byte_identical`, `akribie-core/src/emit/svg/tests.rs::svg_is_byte_identical_for_the_same_scene` |
| 3 | PNG/JPG/AVIF exigem `--out` | binário só no arquivo (`--scale` multiplica os pixels, não o `viewBox`) | `{bytes,format,height,path,width,dry_run}` | exit 2 sem `--out` | arquivo criado (bytes conferem) | `akribie-cli/tests/render.rs::png_scales_and_requires_out`, `::json_reports_the_written_size`, `::unknown_format_is_usage_error` |
| 4 | `--dry-run` mede sem gravar | `<bytes> B` e nada mais | `dry_run:true`, `bytes` | exit 0 | nada escrito | `akribie-cli/tests/render.rs::dry_run_reports_bytes_without_writing` |
| 5 | `--params` escolhe entrada e saída | com `out` o SVG vai ao arquivo; stdout vazio | envelope/`data` do formato | exit 2 junto de posicional | arquivo criado | `akribie-cli/tests/render.rs::params_choose_input_and_output` |
| 6 | `--batch` + `--out-dir` | `<alvo>  ok  <bytes> B` / `<alvo>  erro  <CODE> <msg>` | envelope `{ok,failed,items[]}` com `out`/`bytes` por item | exit 0 se ao menos um renderiza; 8 se todos; 3 só quando `--out-dir` não existir (o padrão é `.`); 2 lote vazio ou `--params` | item grava `<name>.<ext do item>` (ou `out` do item); `--dry-run` não grava | `akribie-cli/tests/render.rs::batch_writes_only_valid_items_and_stays_zero_when_some_succeed`, `::batch_honors_per_item_format_and_scale`, `::batch_requires_existing_out_dir_and_rejects_empty_or_conflicting_use`, `::batch_json_counts_items_and_reports_targets`, `::batch_dry_run_writes_nothing` |
| 7 | `--at REV` via Git | renderiza a fonte do commit (`git show`) | idem, no envelope | exit 3 com revisão inexistente | nada escrito | `akribie-cli/tests/render.rs::at_revision_renders_the_committed_source` |
| 8 | `scene` em stdout | JSON determinístico, `\n` final | `{bytes,format}` | exit 8 com erro | nada escrito | `akribie-cli/tests/render.rs::scene_format_is_json_on_stdout`; `akribie-core/src/emit/scene_json/tests.rs` |
| 9 | `excalidraw` em stdout | JSON bonito, `index` `a0000V`+ e `endArrowhead` | `{bytes,format}` | exit 8 com erro | nada escrito | `akribie-cli/tests/render.rs::excalidraw_export_is_json_with_order_keys`; `akribie-core/src/emit/excalidraw/tests.rs::elements_are_ordered_and_indexed`, `::bindings_exist_and_never_use_exact_half` |
| 10 | fonte inválida | diagnósticos em stdout | envelope `failed` | exit 8, nada escrito | inalterado | `akribie-cli/tests/render.rs::invalid_source_is_not_rendered` |

## `akr import`

| # | caso | pipe | `--json` | erro+exit | estado/arquivo | teste |
|---|---|---|---|---|---|---|
| 1 | `.excalidraw` → `.akrc` | fonte canônica em `stdout`; perdas em `stderr` | `data.{source,diagnostics}` | exit 0; `--params` traz `path`/`source`/`name` (título) | nada escrito | `akribie-cli/tests/import.rs::excalidraw_becomes_canonical_akrc_accepted_by_check`, `::import_from_a_file_keeps_the_diagnostics_in_the_envelope`; `akribie-core/src/import/excalidraw/tests.rs::maps_shapes_text_arrows_and_reports_losses`, `::the_result_passes_check`, `::free_text_becomes_a_note`; `akribie-cli/tests/import.rs::params_choose_the_source_and_the_title`, `::params_plus_positional_is_usage_error` |
| 2 | JSON inválido | mensagem em `stderr` | `error.code:"invalid_input"` | exit 2 | nada escrito | `akribie-cli/tests/import.rs::invalid_json_is_invalid_input`, `::wrong_document_shape_is_invalid_input`; `akribie-core/src/import/excalidraw/tests.rs::rejects_foreign_json` |

## `akr diff`

| # | caso | pipe | `--json` | erro+exit | estado/arquivo | teste |
|---|---|---|---|---|---|---|
| 1 | três mudanças em três linhas | `+`/`-`/`~`, ordem added→removed→changed | `{added,removed,changed,old,new}` | exit 1 quando difere, 0 igual | — | `akribie-cli/tests/diff.rs::three_changes_are_three_lines_in_order`, `::json_groups_changes_by_kind`, `::identical_files_are_silent_and_succeed`; `akribie-core/src/diff/tests.rs` |
| 3 | fonte inválida de um lado | diagnósticos daquele arquivo em stdout | envelope `failed` | exit 8, nada comparado | — | `akribie-cli/tests/diff.rs::invalid_side_blocks_the_diff` |
| 4 | posicionais ausentes / `- -` | — | — | exit 2 | — | `akribie-cli/tests/diff.rs::missing_positionals_are_usage_errors`, `::both_sides_from_stdin_is_a_usage_error` |
| 2 | `--at REV` (flag ou `--params.at`) | compara `REV:FILE` com a árvore de trabalho | `{old,new,added,removed,changed}` | exit 1 quando difere; 3 com revisão inexistente; 2 com dois lados | — | `akribie-cli/tests/diff.rs::at_revision_compares_the_working_tree_with_the_commit`, `::at_with_unknown_revision_is_not_found`, `::params_at_compares_the_revision_with_the_working_tree`, `::params_with_both_sides_and_at_is_usage_error` |

## `akr schema`

| # | caso | pipe | `--json` | erro+exit | estado/arquivo | teste |
|---|---|---|---|---|---|---|
| 1 | lista statements, chaves, valores, temas (canvas/ink/paleta), mínimos de contraste e códigos | JSON pretty + `\n` | envelope próprio (não é `--json`) | — | — | `akribie-cli/tests/check.rs::schema_exposes_closed_sets_and_codes` |

## Transversais

| # | caso | pipe | `--json` | erro+exit | estado/arquivo | teste |
|---|---|---|---|---|---|---|
| T1 | EPIPE em stdout sai 0 | exit 0 | — | EPIPE ⇒ 0 | — | `akribie-cli/tests/check.rs::broken_pipe_is_success` |
| T2 | `--params` substitui flags | mesma saída do verbo | — | exit 0 | — | `akribie-cli/tests/check.rs::params_source_replaces_positionals`, `::params_unknown_key_lists_valid_keys`, `::params_path_and_source_are_exclusive` |
| T3 | `--params` + posicional = exit 2 | — | — | exit 2 | nada escrito | `akribie-cli/tests/check.rs::params_plus_positional_is_usage_error` |
| T4 | contraste WCAG de todas as combinações | — | — | `W007` + exit 8 sob `--strict` | — | `akribie-core/src/theme/tests.rs::text_meets_aa_on_every_fill`, `::ink_and_muted_strokes_meet_non_text_minimum`, `::colored_strokes_are_accents_below_minimum`; `akribie-core/src/a11y.rs::colored_strokes_warn_but_ink_does_not` |
| T5 | SVG válido em `xmllint` + `role`/`title`/`desc` | `xmllint --noout` ok | — | — | — | `akribie-cli/tests/render.rs::svg_goes_to_stdout_with_accessible_marks`; verificado à mão com `xmllint` |
| T6 | MCP: `tools/list` e `chart_check` inválido | 3 ferramentas; inválido ⇒ `isError: true` com a 1ª linha do diagnóstico; servidor segue atendendo | — | falha de tool é dado; `-32700`/`-32601`/`-32602` para protocolo | — | `akribie-mcp/tests/mcp.rs::tools_list_announces_the_three_tools`, `::invalid_source_is_is_error_and_the_server_stays_alive`, `::malformed_json_is_a_protocol_error_without_killing_the_server`, `::render_png_returns_an_image_content_block`, `::diff_reports_the_label_change` |
| T7 | `akr prime` × `docs/usage` em paridade de flags | toda flag de `--help` aparece no guia do verbo; toda flag citada existe em algum verbo | — | — | — | `akribie-cli/tests/docs.rs::help_and_usage_docs_agree_on_flags`, `::every_documented_usage_file_exists` |
| T8 | rótulos não colidem nos diagramas reais | nenhum rótulo de aresta toca nó, outro rótulo ou a borda do canvas; título de grupo cabe inteiro | — | — | quatro `.akrc` de `fixtures/reais/` byte-idênticos ao golden | `akribie-core/src/layout/overlap_tests.rs::edge_labels_clear_nodes_and_each_other`, `::group_labels_fit_inside_the_group`, `::labels_stay_inside_the_canvas`; `akribie-cli/tests/render.rs::real_world_diagrams_match_goldens`; `akribie-cli/tests/docs.rs::example_images_exist` |
| T9 | `=` dentro de string entre aspas é conteúdo | rótulo sai intacto no SVG | — | exit 0 | — | `akribie-core/src/lex/tests.rs::quoted_label_keeps_equals_as_content`, `::quoted_edge_label_keeps_equals_as_content`, `::unquoted_option_still_splits_on_equals`; `akribie-cli/tests/check.rs::quoted_label_may_contain_equals` |
