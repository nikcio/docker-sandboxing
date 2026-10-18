//! Shared HTTP client for the discovery APIs, with a bounded response
//! cache and size limit.

use std::collections::HashMap;
use std::io::Read;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::MAX_BODY;
use super::descriptor::is_kit_tag;
use reqwest::blocking::Client;
use serde_json::Value;

#[derive(Clone)]
pub(super) struct Api {
    pub(super) client: Client,
    pub(super) github: String,
    pub(super) raw: String,
    pub(super) hub: String,
    pub(super) registry: String,
    pub(super) auth: String,
}

impl Api {
    pub(super) fn public() -> Result<Self, String> {
        Ok(Self {
            client: Client::builder()
                .user_agent("Sandy/0.1 (public sandbox-kit discovery)")
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|e| e.to_string())?,
            github: "https://api.github.com".into(),
            raw: "https://raw.githubusercontent.com".into(),
            hub: "https://hub.docker.com".into(),
            registry: "https://registry-1.docker.io".into(),
            auth: "https://auth.docker.io/token".into(),
        })
    }

    pub(super) fn text(&self, url: &str) -> Result<String, String> {
        self.request(url, None).map(|(text, _)| text)
    }

    pub(super) fn json(&self, url: &str) -> Result<Value, String> {
        serde_json::from_str(&self.text(url)?).map_err(|e| format!("Invalid JSON from {url}: {e}"))
    }

    pub(super) fn request(
        &self,
        url: &str,
        token: Option<&str>,
    ) -> Result<(String, Option<String>), String> {
        // Cache successes only; a rate limit or missing repository must remain visible.
        type HttpCache = Mutex<HashMap<String, (Instant, String, Option<String>)>>;
        static CACHE: OnceLock<HttpCache> = OnceLock::new();
        let cache = CACHE.get_or_init(Default::default);
        if let Some((at, body, digest)) = cache.lock().map_err(|e| e.to_string())?.get(url)
            && at.elapsed() < Duration::from_secs(300)
        {
            return Ok((body.clone(), digest.clone()));
        }
        let result = self.request_uncached(url, token);
        crate::diaglog::http_call("GET", url, &result);
        let (body, digest) = result?;
        let mut entries = cache.lock().map_err(|e| e.to_string())?;
        if entries.len() > 1024 {
            entries.retain(|_, (at, _, _)| at.elapsed() < Duration::from_secs(300));
        }
        if entries.len() > 1024 {
            entries.clear();
        }
        entries.insert(url.into(), (Instant::now(), body.clone(), digest.clone()));
        Ok((body, digest))
    }

    fn request_uncached(
        &self,
        url: &str,
        token: Option<&str>,
    ) -> Result<(String, Option<String>), String> {
        let mut request = self.client.get(url);
        if let Some(token) = token {
            request = request.bearer_auth(token).header(
                "Accept",
                concat!(
                    "application/vnd.oci.image.index.v1+json, ",
                    "application/vnd.oci.image.manifest.v1+json, ",
                    "application/vnd.docker.distribution.manifest.list.v2+json, ",
                    "application/vnd.docker.distribution.manifest.v2+json"
                ),
            );
        }
        let response = request.send().map_err(|e| format!("{url}: {e}"))?;
        let status = response.status();
        if !status.is_success() {
            let explanation = if status.as_u16() == 403 || status.as_u16() == 429 {
                " (public API rate limit or access policy; retry later)"
            } else {
                ""
            };
            return Err(format!("{url}: HTTP {status}{explanation}"));
        }
        let digest = response
            .headers()
            .get("docker-content-digest")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let mut body = String::new();
        response
            .take(MAX_BODY + 1)
            .read_to_string(&mut body)
            .map_err(|e| e.to_string())?;
        if body.len() as u64 > MAX_BODY {
            return Err(format!("Metadata from {url} exceeds 12 MiB"));
        }
        Ok((body, digest))
    }

    pub(super) fn github_tags(&self, repository: &str) -> Result<Vec<Value>, String> {
        let mut tags = Vec::new();
        for page in 1..=100 {
            let value = self.json(&format!(
                "{}/repos/{repository}/tags?per_page=100&page={page}",
                self.github
            ))?;
            let entries = value
                .as_array()
                .ok_or("GitHub tags response is not an array")?;
            tags.extend(entries.iter().cloned());
            if entries.len() < 100 {
                return Ok(tags);
            }
        }
        Err("GitHub repository has over 10,000 tags; choose a reference directly".into())
    }

    pub(super) fn hub_tags(&self, repository: &str) -> Result<Vec<Value>, String> {
        let mut tags = Vec::new();
        for page in 1..=100 {
            let value = self.json(&format!(
                "{}/v2/repositories/{repository}/tags?page_size=100&page={page}",
                self.hub
            ))?;
            let entries = value["results"]
                .as_array()
                .ok_or("Hub tags response has no results")?;
            tags.extend(entries.iter().filter(|v| is_kit_tag(v)).cloned());
            if value["next"].as_str().is_none_or(str::is_empty) {
                return Ok(tags);
            }
        }
        Err("Docker Hub repository has over 10,000 tags; choose a tag directly".into())
    }
}
