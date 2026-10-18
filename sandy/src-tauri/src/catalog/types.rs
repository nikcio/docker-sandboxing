//! The provider catalog data model: providers, kit options, and the catalog
//! returned by discovery.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provider {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub repository: String,
    #[serde(default)]
    pub reference: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub schema_version: u8,
    pub kind: String,
    pub provider_id: String,
    pub required_agent: Option<String>,
    #[serde(default)]
    pub arguments: Vec<KitArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Catalog {
    pub kits: Vec<KitOption>,
    pub warnings: Vec<String>,
}
