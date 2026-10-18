//! Docker Hub provider discovery: catalog search, typed kit tags, and OCI
//! manifest reads that never pull image layers.

use serde_json::Value;

use super::descriptor::{is_kit_tag, option_from_descriptor};
use super::http::Api;
use super::store::hub_repository;
use super::types::{Catalog, KitOption, Provider};
use super::{DESCRIPTOR, METADATA_BATCH, encoded, encoded_path, repository_name};

pub(super) fn is_kit_repository(value: &Value) -> bool {
    value["content_types"]
        .as_array()
        .is_some_and(|v| v.iter().any(|v| v == "sbx_kit"))
        || value["media_types"].as_array().is_some_and(|v| {
            v.iter().any(|m| {
                m.as_str()
                    .is_some_and(|m| m.starts_with("application/vnd.docker.sandbox.kit.v"))
            })
        })
}

pub(super) fn discover_hub(api: &Api, provider: &Provider, query: &str) -> Result<Catalog, String> {
    let mut catalog = Catalog::default();
    let repositories = if let Some(repository) = hub_repository(&provider.repository)? {
        vec![repository]
    } else {
        // The Hub catalog API uses from/size (not page_size). Keep browsing bounded and ask for a narrower search.
        let value = api.json(&format!(
            "{}/api/search/v3/catalog/search?type=sbx_kit&query={}&from=0&size=24",
            api.hub,
            encoded(query)
        ))?;
        let entries = value["results"]
            .as_array()
            .ok_or("Hub catalog response has no results")?;
        if value["total"].as_u64().unwrap_or(0) > entries.len() as u64 {
            catalog.warnings.push(format!("{}: showing the first {} matching repositories; search to narrow results or add a repository provider to choose its tags", provider.label, entries.len()));
        }
        entries
            .iter()
            .filter(|v| is_kit_repository(v))
            .filter_map(|v| v["slug"].as_str().or_else(|| v["id"].as_str()))
            .map(String::from)
            .collect()
    };
    for chunk in repositories.chunks(METADATA_BATCH) {
        let jobs = std::thread::scope(|scope| {
            chunk
                .iter()
                .map(|repository| {
                    scope.spawn(move || (repository, selected_hub_kit(api, provider, repository)))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|job| {
                    job.join()
                        .map_err(|_| "Hub metadata worker failed".to_string())
                })
                .collect::<Vec<_>>()
        });
        for job in jobs {
            let (repository, result) = job?;
            match result {
                Ok(kit) => catalog.kits.push(kit),
                Err(e) => catalog
                    .warnings
                    .push(format!("{} / {repository}: {e}", provider.label)),
            }
        }
    }
    Ok(catalog)
}

pub(super) fn selected_hub_kit(
    api: &Api,
    provider: &Provider,
    repository: &str,
) -> Result<KitOption, String> {
    if let Some(digest) = provider.reference.strip_prefix("sha256:") {
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(
                "An OCI sha256 digest must contain exactly 64 hexadecimal characters".into(),
            );
        }
        let source = format!("docker.io/{repository}@{}", provider.reference);
        let descriptor = hub_descriptor(api, repository, &provider.reference)?;
        return option_from_descriptor(
            &descriptor,
            &source,
            &provider.id,
            repository_name(repository).unwrap_or(repository),
        );
    }
    let response = if provider.reference.is_empty() {
        api.json(&format!(
            "{}/v2/repositories/{repository}/tags?page_size=100&page=1",
            api.hub
        ))?
    } else {
        let tag = api.json(&format!(
            "{}/v2/repositories/{repository}/tags/{}",
            api.hub,
            encoded_path(&provider.reference)
        ))?;
        serde_json::json!({"results":[tag]})
    };
    let tag = response["results"]
        .as_array()
        .and_then(|tags| tags.iter().find(|tag| is_kit_tag(tag)))
        .ok_or_else(|| format!("No sandbox-kit tag {} in the returned tags for {repository}; add this repository as a provider to select a version", provider.reference))?;
    let name = tag["name"].as_str().ok_or("Hub tag has no name")?;
    // Retain the chosen tag in the reference for usability. Metadata reads are cached.
    let source = format!("docker.io/{repository}:{name}");
    let descriptor = hub_descriptor(api, repository, name)?;
    option_from_descriptor(
        &descriptor,
        &source,
        &provider.id,
        repository_name(repository).unwrap_or(repository),
    )
}

pub(super) fn hub_descriptor(
    api: &Api,
    repository: &str,
    reference: &str,
) -> Result<Value, String> {
    let token = api.json(&format!(
        "{}?service=registry.docker.io&scope=repository:{}:pull",
        api.auth,
        encoded_path(repository)
    ))?["token"]
        .as_str()
        .ok_or("Docker registry returned no public pull token")?
        .to_string();
    let mut reference = reference.to_string();
    for _ in 0..4 {
        let (body, _) = api.request(
            &format!(
                "{}/v2/{repository}/manifests/{}",
                api.registry,
                encoded_path(&reference)
            ),
            Some(&token),
        )?;
        let manifest: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if let Some(descriptor) = manifest["annotations"][DESCRIPTOR].as_str() {
            return serde_yaml_ng::from_str(descriptor)
                .map_err(|e| format!("Invalid OCI kit descriptor: {e}"));
        }
        if let Some(manifests) = manifest["manifests"].as_array() {
            reference = manifests
                .iter()
                .find(|m| m["platform"]["os"] == "linux")
                .or_else(|| manifests.first())
                .and_then(|m| m["digest"].as_str())
                .ok_or("OCI index has no image manifest")?
                .into();
            continue;
        }
        // V2's artifactType is application/vnd.docker.sandbox.kit.v2, and its
        // config is the YAML spec (not an image config). Some older publishers
        // used a spec layer instead. Neither requires pulling image content.
        if manifest["config"]["mediaType"]
            .as_str()
            .is_some_and(|m| m.ends_with(".spec+yaml"))
        {
            let digest = manifest["config"]["digest"]
                .as_str()
                .ok_or("Kit spec config has no digest")?;
            let (text, _) = api.request(
                &format!("{}/v2/{repository}/blobs/{digest}", api.registry),
                Some(&token),
            )?;
            return serde_yaml_ng::from_str(&text).map_err(|e| e.to_string());
        }
        // Read only the small spec layer, never filesystem/image layers.
        if let Some(layers) = manifest["layers"].as_array()
            && let Some(layer) = layers.iter().find(|l| {
                l["mediaType"]
                    .as_str()
                    .is_some_and(|m| m.ends_with(".spec+yaml"))
            })
        {
            let digest = layer["digest"]
                .as_str()
                .ok_or("Kit spec layer has no digest")?;
            let (text, _) = api.request(
                &format!("{}/v2/{repository}/blobs/{digest}", api.registry),
                Some(&token),
            )?;
            return serde_yaml_ng::from_str(&text).map_err(|e| e.to_string());
        }
        // Older published formats are understood by the installed CLI. Only bounded selected tags reach this fallback.
        let separator = if reference.starts_with("sha256:") {
            "@"
        } else {
            ":"
        };
        let output = crate::sbx::run_capture(&[
            "kit",
            "inspect",
            &format!("docker.io/{repository}{separator}{reference}"),
            "--json",
        ])?;
        return serde_json::from_str(&output)
            .map_err(|e| format!("Invalid sbx kit inspect JSON: {e}"));
    }
    Err("OCI image index nesting exceeds four levels".into())
}
