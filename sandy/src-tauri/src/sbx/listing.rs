//! `SandboxInfo` and parsing of `sbx ls` output (JSON with a human-table
//! fallback for older CLI versions).

use serde::Serialize;

use super::cli::run_capture;

#[derive(Debug, Serialize, Clone)]
pub struct SandboxInfo {
    pub name: String,
    pub state: String,
    pub agent: String,
    pub workspace: String,
    pub raw: String,
}

/// List local sandboxes. Prefers `sbx ls --json` (stable, machine-readable)
/// and falls back to parsing the human table if JSON is unavailable or
/// entirely unparseable (older CLI versions).
pub fn list_sandboxes() -> Result<Vec<SandboxInfo>, String> {
    let json_result = run_capture(&["ls", "--json"]).and_then(|out| {
        // Tolerate individual bad entries instead of discarding the whole
        // payload: parse as raw values, keep the ones that deserialize.
        let values: Vec<serde_json::Value> = serde_json::from_str(&out)
            .map_err(|e| format!("could not parse sbx ls --json output: {}", e))?;
        Ok(values
            .into_iter()
            .filter_map(|v| serde_json::from_value::<SandboxJson>(v).ok())
            .map(|s| SandboxInfo {
                name: s.name,
                state: s.status.unwrap_or_default(),
                agent: s.agent.unwrap_or_default(),
                workspace: s.workspaces.unwrap_or_default().join(", "),
                raw: s.id.unwrap_or_default(),
            })
            .collect())
    });
    match json_result {
        Ok(rows) => Ok(rows),
        Err(json_err) => match run_capture(&["ls"]) {
            Ok(out) if !out.is_empty() => Ok(parse_ls(&out)),
            Ok(_) => Err(json_err),
            Err(_) => Err(json_err),
        },
    }
}

/// Schema of `sbx ls --json` entries. Unknown fields are ignored; optional
/// fields degrade to empty so a schema addition never breaks the UI.
#[derive(Debug, serde::Deserialize)]
struct SandboxJson {
    name: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    workspaces: Option<Vec<String>>,
}

pub(crate) fn parse_ls(out: &str) -> Vec<SandboxInfo> {
    let mut rows = Vec::new();
    let mut header_cols: Option<Vec<(usize, usize)>> = None;
    // Maps logical columns (name/state/agent/workspace) to their index in
    // the header row, since `sbx ls` column names and order vary.
    let mut column_map: Option<std::collections::HashMap<&'static str, usize>> = None;
    // Box-drawing tables carry interior pipes; once the header has them we
    // split data rows on pipes directly instead of header word spans.
    let mut pipe_table = false;

    for line in out.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() || is_separator_line(trimmed) {
            continue;
        }
        // Only the first table header is recognized; later lines that happen
        // to look like headers stay data rows.
        if header_cols.is_none() && is_header_line(trimmed) {
            pipe_table = trimmed.contains('│') || trimmed.contains('|');
            let header = strip_border_pipes(trimmed).replace(['│', '|'], " ");
            let spans = header_spans(&header);
            let names = header_spans(&header)
                .iter()
                .map(|(s, e)| header[*s..*e].to_uppercase())
                .collect::<Vec<_>>();
            column_map = Some(map_columns(&names));
            header_cols = Some(spans);
            continue;
        }
        let cols = if pipe_table {
            strip_border_pipes(trimmed)
                .split(['│', '|'])
                .map(|c| c.trim().to_string())
                .collect()
        } else {
            match &header_cols {
                Some(spans) => split_by_spans(trimmed, spans),
                None => vec![trimmed.trim().to_string()],
            }
        };
        let col = |name: &str| -> String {
            match column_map.as_ref().and_then(|m| m.get(name).copied()) {
                Some(i) => cols
                    .get(i)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default(),
                // No recognized header: treat the whole line as the name so
                // single-line output still shows something sensible.
                None if name == "name" => cols
                    .first()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default(),
                None => String::new(),
            }
        };
        rows.push(SandboxInfo {
            name: col("name"),
            state: col("state"),
            agent: col("agent"),
            workspace: col("workspace"),
            raw: trimmed.to_string(),
        });
    }
    rows
}

fn is_header_line(line: &str) -> bool {
    let lower = line
        .trim()
        .trim_start_matches(['│', '|'])
        .trim()
        .to_lowercase();
    // Known header shapes: the original NAME/STATE/AGENT/WORKSPACE layout
    // and the real v0.48 SANDBOX/AGENT/STATUS/PORTS/WORKSPACE/CREATED one.
    (lower.starts_with("name") && lower.contains("state"))
        || (lower.starts_with("sandbox") && lower.contains("status"))
}

/// Map header column names to the logical fields the UI shows. Recognizes
/// both the SANDBOX/STATUS and NAME/STATE vocabularies, plus aliases.
fn map_columns(names: &[String]) -> std::collections::HashMap<&'static str, usize> {
    let mut map = std::collections::HashMap::new();
    for (i, raw) in names.iter().enumerate() {
        let name = raw.trim();
        match name {
            "NAME" | "SANDBOX" => {
                map.entry("name").or_insert(i);
            }
            "STATE" | "STATUS" => {
                map.entry("state").or_insert(i);
            }
            "AGENT" => {
                map.entry("agent").or_insert(i);
            }
            "WORKSPACE" | "WORKSPACES" => {
                map.entry("workspace").or_insert(i);
            }
            _ => {}
        }
    }
    map
}

/// Remove a leading and trailing border pipe from a box-drawing table row
/// and trim the result.
fn strip_border_pipes(line: &str) -> &str {
    let t = line.trim();
    let stripped = t
        .trim_start_matches(['│', '|'])
        .trim_end_matches(['│', '|']);
    stripped.trim_end()
}

/// True for table decoration lines (┌───┬───┐, ├───┼───┤, +----+----+, ───).
fn is_separator_line(line: &str) -> bool {
    let mut has_separator = false;
    for c in line.trim().chars() {
        if c.is_whitespace() {
            continue;
        }
        if matches!(
            c,
            '─' | '━'
                | '-'
                | '='
                | '+'
                | '┌'
                | '┐'
                | '└'
                | '┘'
                | '├'
                | '┤'
                | '┬'
                | '┴'
                | '┼'
                | '│'
                | '|'
        ) {
            has_separator = true;
        } else {
            return false;
        }
    }
    has_separator
}

/// Find the byte spans of each header word so data rows can be split on the
/// same columns.
fn header_spans(header: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in header.char_indices() {
        if c == ' ' {
            if let Some(s) = start.take() {
                spans.push((s, i));
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        spans.push((s, header.len()));
    }
    spans
}

fn split_by_spans(line: &str, spans: &[(usize, usize)]) -> Vec<String> {
    // Column boundaries are the header words' start offsets. A cell's value
    // is often wider than its header, so a cell ends at the last
    // non-whitespace before the next column's start — not at the next
    // header word's start itself.
    let mut out = Vec::new();
    for (i, (s, _e)) in spans.iter().enumerate() {
        let next_start = if i + 1 < spans.len() {
            spans[i + 1].0
        } else {
            line.len()
        };
        let mut end = next_start.min(line.len());
        while end > *s && end > 0 && line[..end].ends_with(' ') {
            end -= 1;
        }
        out.push(line.get(*s..end).unwrap_or("").trim().to_string());
    }
    out
}
