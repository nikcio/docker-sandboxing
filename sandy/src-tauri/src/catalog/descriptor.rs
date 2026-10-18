//! Kit descriptor parsing: turning v2/v3 YAML descriptors into kit options.

use std::collections::BTreeMap;

use serde_json::Value;

use super::types::{KitArgument, KitOption};

pub(super) fn comment_descriptor(text: &str) -> Option<String> {
    let mut in_block = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.trim() == "# kit:" {
            in_block = true;
            continue;
        }
        if in_block {
            if let Some(line) = line.strip_prefix("#   ") {
                lines.push(line);
            } else if line.trim() == "#" {
                lines.push("");
            } else {
                break;
            }
        }
    }
    in_block.then(|| lines.join("\n"))
}

pub(super) fn schema_version(value: &Value) -> Option<u8> {
    value["schemaVersion"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            value["schemaVersion"]
                .as_u64()
                .and_then(|n| n.try_into().ok())
        })
}

pub(super) fn option_from_descriptor(
    value: &Value,
    source: &str,
    provider_id: &str,
    fallback: &str,
) -> Result<KitOption, String> {
    let schema_version = schema_version(value)
        .filter(|v| *v == 2 || *v == 3)
        .ok_or("Only kit schemas 2 and 3 are supported")?;
    let kind = match (schema_version, value["kind"].as_str()) {
        (2, Some("sandbox")) => "agent",
        (3, Some("sandbox" | "workload")) => "workload",
        (_, Some("mixin")) => "mixin",
        (_, Some("set")) => return Err("Authored kit sets must be published before selection (their derived workload/mixin kind is not yet known)".into()),
        _ => return Err("Unsupported kit kind".into()),
    };
    let name = if schema_version == 2 {
        value["name"]
            .as_str()
            .ok_or("V2 kit descriptor must declare its name")?
    } else {
        value["name"].as_str().unwrap_or(fallback)
    }
    .to_string();
    if name.is_empty() {
        return Err("Kit has no identity".into());
    }
    let args: BTreeMap<String, Value> = value
        .get("args")
        .and_then(Value::as_object)
        .map(|args| {
            args.iter()
                .map(|(name, arg)| (name.clone(), arg.clone()))
                .collect()
        })
        .unwrap_or_default();
    let arguments = args
        .into_iter()
        .map(|(name, arg)| KitArgument {
            name,
            description: arg["description"].as_str().unwrap_or_default().into(),
            required: arg["required"].as_bool().unwrap_or(false),
            default: arg["default"].as_str().map(String::from),
            choices: arg["enum"]
                .as_array()
                .map(|v| {
                    v.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect();
    Ok(KitOption {
        id: format!("{provider_id}:{source}"),
        name,
        source: source.into(),
        schema_version,
        kind: kind.into(),
        provider_id: provider_id.into(),
        description: value["description"]
            .as_str()
            .or_else(|| value["displayName"].as_str())
            .unwrap_or_default()
            .into(),
        required_agent: value["requires"]["agent"].as_str().map(String::from),
        arguments,
    })
}

pub(super) fn is_kit_tag(value: &Value) -> bool {
    value["content_type"] == "sbx_kit"
        || value["media_type"]
            .as_str()
            .is_some_and(|m| m.starts_with("application/vnd.docker.sandbox.kit.v"))
}
