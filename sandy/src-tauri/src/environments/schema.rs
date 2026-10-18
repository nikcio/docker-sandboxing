//! The environment YAML document model (schema "1") and value escaping.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Environment {
    pub(super) schema_version: String,
    pub(super) name: String,
    pub(super) agent: String,
    pub(super) workspace: Workspace,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) kits: Vec<KitEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum Workspace {
    Path(String),
    Object {
        path: String,
        #[serde(default)]
        clone: bool,
    },
}

impl Workspace {
    pub(super) fn path(&self) -> String {
        unescape(match self {
            Self::Path(path) | Self::Object { path, .. } => path,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum KitEntry {
    Source(String),
    WithArgs {
        source: String,
        args: BTreeMap<String, String>,
    },
}

impl KitEntry {
    pub(super) fn source(&self) -> &str {
        match self {
            Self::Source(source) | Self::WithArgs { source, .. } => source,
        }
    }

    pub(super) fn arguments(&self) -> BTreeMap<String, String> {
        match self {
            Self::Source(_) => BTreeMap::new(),
            Self::WithArgs { args, .. } => {
                args.iter().map(|(k, v)| (k.clone(), unescape(v))).collect()
            }
        }
    }
}

pub(super) fn escape(value: &str) -> String {
    value.replace("${{", "$${{")
}

fn unescape(value: &str) -> String {
    value.replace("$${{", "${{")
}
