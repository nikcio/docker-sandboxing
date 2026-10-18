use std::io::{Read, Write};
use std::net::TcpListener;

use super::descriptor::{is_kit_tag, option_from_descriptor};
use super::http::Api;
use super::hub::is_kit_repository;
use super::hub::{hub_descriptor, selected_hub_kit};
use super::types::Provider;
use super::{
    BuiltinAgents, DESCRIPTOR, builtin_agents, discover_with, encoded_path, get_providers,
    parse_available_agents, save_providers, unique_id, versions_with,
};
fn fixture(routes: Vec<(&str, &str)>) -> (Api, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let routes: Vec<_> = routes
        .into_iter()
        .map(|(p, b)| (p.to_string(), b.to_string()))
        .collect();
    let worker = std::thread::spawn(move || {
        for (path, body) in routes {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let n = stream.read(&mut request).unwrap();
            assert!(String::from_utf8_lossy(&request[..n]).starts_with(&format!("GET {path} ")));
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        }
    });
    let mut api = Api::public().unwrap();
    api.github = base.clone();
    api.raw = base.clone();
    api.hub = base.clone();
    api.registry = base.clone();
    api.auth = format!("{base}/token");
    (api, worker)
}

#[test]
fn github_api_resolves_selected_tag_and_discovers_descriptor() {
    let (api, worker) = fixture(vec![
        ("/repos/team/kits/commits/v1.0", r#"{"sha":"abc123"}"#),
        (
            "/repos/team/kits/git/trees/abc123?recursive=1",
            r#"{"tree":[{"type":"blob","path":"tool/spec.yaml"}]}"#,
        ),
        (
            "/team/kits/abc123/tool/spec.yaml",
            "schemaVersion: '2'\nkind: mixin\nname: tool\nrequires: {agent: claude}\nargs:\n  channel:\n    required: true\n    enum: [stable, beta]\n",
        ),
    ]);
    let provider = Provider {
        id: "custom".into(),
        label: "Test".into(),
        kind: "github".into(),
        repository: "team/kits".into(),
        reference: "v1.0".into(),
        enabled: true,
    };
    let result = discover_with(&api, &[provider], "").unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    assert_eq!(result.kits.len(), 1);
    assert_eq!(
        result.kits[0].source,
        "git+https://github.com/team/kits.git#ref=abc123&dir=tool"
    );
    assert_eq!(result.kits[0].required_agent.as_deref(), Some("claude"));
    assert_eq!(result.kits[0].arguments[0].choices, ["stable", "beta"]);
    worker.join().unwrap();
}

#[test]
fn manual_github_branch_and_commit_do_not_list_tags() {
    for reference in ["feature/next", "abc123"] {
        // The fixture has no tag endpoint. An attempt to list tags fails
        // the expected-request assertion instead of masking the bug.
        let commit_path = format!("/repos/team/kits/commits/{}", encoded_path(reference));
        let (api, worker) = fixture(vec![
            (&commit_path, r#"{"sha":"abc123"}"#),
            (
                "/repos/team/kits/git/trees/abc123?recursive=1",
                r#"{"tree":[{"type":"blob","path":"tool/spec.yaml"}]}"#,
            ),
            (
                "/team/kits/abc123/tool/spec.yaml",
                "schemaVersion: '2'\nkind: mixin\nname: tool\n",
            ),
        ]);
        let provider = Provider {
            id: "manual".into(),
            label: "Manual reference".into(),
            kind: "github".into(),
            repository: "team/kits".into(),
            reference: reference.into(),
            enabled: true,
        };
        let catalog = discover_with(&api, &[provider], "").unwrap();
        assert!(catalog.warnings.is_empty(), "{:?}", catalog.warnings);
        assert_eq!(catalog.kits.len(), 1);
        assert!(catalog.kits[0].source.contains("#ref=abc123&dir=tool"));
        worker.join().unwrap();
    }
}

#[test]
fn normalized_v3_and_typed_filtering() {
    let value = serde_json::json!({"schemaVersion":"3","kind":"sandbox","name":"sbx-kit-claude"});
    assert_eq!(
        option_from_descriptor(
            &value,
            "docker.io/docker/sbx-kit-claude:1",
            "hub",
            "fallback"
        )
        .unwrap()
        .kind,
        "workload"
    );
    let value = serde_json::json!({"schemaVersion":"3","kind":"workload"});
    assert_eq!(
        option_from_descriptor(&value, "source", "id", "alpine")
            .unwrap()
            .name,
        "alpine"
    );
    assert!(!is_kit_repository(
        &serde_json::json!({"content_types":["image"]})
    ));
    assert!(is_kit_repository(
        &serde_json::json!({"content_types":["image","sbx_kit"]})
    ));
    assert!(!is_kit_tag(
        &serde_json::json!({"name":"latest","content_type":"image"})
    ));
    assert!(is_kit_tag(
        &serde_json::json!({"media_type":"application/vnd.docker.sandbox.kit.v2"})
    ));
    assert!(
        option_from_descriptor(
            &serde_json::json!({"schemaVersion":"1","kind":"mixin"}),
            "s",
            "p",
            "n"
        )
        .is_err()
    );
}

#[test]
fn hub_versions_filter_plain_images_and_keep_tag_selection() {
    let (api, worker) = fixture(vec![(
        "/v2/repositories/team/kit/tags?page_size=100&page=1",
        r#"{"next":null,"results":[{"name":"plain","content_type":"image"},{"name":"v3","content_type":"sbx_kit"},{"name":"v2","media_type":"application/vnd.docker.sandbox.kit.v2"}]}"#,
    )]);
    let provider = Provider {
        id: "hub".into(),
        label: "Test".into(),
        kind: "dockerhub".into(),
        repository: "team/kit".into(),
        reference: String::new(),
        enabled: true,
    };
    assert_eq!(versions_with(&api, &provider).unwrap(), ["v3", "v2"]);
    worker.join().unwrap();
}

#[test]
fn oci_v2_reads_spec_config_without_cli_or_image_layers() {
    let (api, worker) = fixture(vec![
        (
            "/token?service=registry.docker.io&scope=repository:team%2Fkit:pull",
            r#"{"token":"public-fixture-token"}"#,
        ),
        (
            "/v2/team/kit/manifests/1.0",
            r#"{"artifactType":"application/vnd.docker.sandbox.kit.v2","config":{"mediaType":"application/vnd.docker.sandbox.kit.v2.spec+yaml","digest":"sha256:abc"},"layers":[{"mediaType":"application/vnd.oci.empty.v1+json"}]}"#,
        ),
        (
            "/v2/team/kit/blobs/sha256:abc",
            "schemaVersion: '2'\nkind: sandbox\nname: custom\n",
        ),
    ]);
    let descriptor = hub_descriptor(&api, "team/kit", "1.0").unwrap();
    assert_eq!(
        option_from_descriptor(&descriptor, "source", "hub", "kit")
            .unwrap()
            .kind,
        "agent"
    );
    worker.join().unwrap();
}

#[test]
fn immutable_hub_reference_skips_tag_lookup_and_preserves_digest() {
    let digest = format!("sha256:{}", "a".repeat(64));
    let path = format!("/v2/team/kit/manifests/{}", encoded_path(&digest));
    let descriptor = serde_json::json!({"schemaVersion":"3","kind":"workload"});
    let manifest =
        serde_json::json!({"annotations": {DESCRIPTOR: descriptor.to_string()}}).to_string();
    let (api, worker) = fixture(vec![
        (
            "/token?service=registry.docker.io&scope=repository:team%2Fkit:pull",
            r#"{"token":"public-fixture-token"}"#,
        ),
        (&path, &manifest),
    ]);
    let mut provider = Provider {
        id: "hub".into(),
        label: "Test".into(),
        kind: "dockerhub".into(),
        repository: "team/kit".into(),
        reference: digest.clone(),
        enabled: true,
    };
    let kit = selected_hub_kit(&api, &provider, "team/kit").unwrap();
    assert_eq!(kit.source, format!("docker.io/team/kit@{digest}"));
    assert_eq!(kit.kind, "workload");
    worker.join().unwrap();
    provider.reference = "sha256:invalid".into();
    assert!(
        selected_hub_kit(&api, &provider, "team/kit")
            .unwrap_err()
            .contains("64 hexadecimal")
    );
}

#[test]
fn oci_v3_index_reads_annotated_descriptor_without_layers() {
    let descriptor = serde_json::json!({"schemaVersion":"3","kind":"workload","args":{"team":{"required":true,"enum":["alpha","beta"]}}});
    let manifest = serde_json::json!({"annotations": {DESCRIPTOR: descriptor.to_string()}, "layers":[{"digest":"never-download"}]}).to_string();
    let (api, worker) = fixture(vec![
        (
            "/token?service=registry.docker.io&scope=repository:team%2Fkit:pull",
            r#"{"token":"public-fixture-token"}"#,
        ),
        (
            "/v2/team/kit/manifests/3.0",
            r#"{"manifests":[{"digest":"sha256:child","platform":{"os":"linux"}}]}"#,
        ),
        ("/v2/team/kit/manifests/sha256%3Achild", &manifest),
    ]);
    let descriptor = hub_descriptor(&api, "team/kit", "3.0").unwrap();
    let kit = option_from_descriptor(&descriptor, "source", "hub", "kit").unwrap();
    assert_eq!(kit.kind, "workload");
    assert!(kit.arguments[0].required);
    assert_eq!(kit.arguments[0].choices, ["alpha", "beta"]);
    worker.join().unwrap();
}

#[test]
fn provider_failure_preserves_other_provider_results() {
    let (api, worker) = fixture(vec![
        (
            "/repos/broken/kits/tags?per_page=100&page=1",
            "invalid-json",
        ),
        (
            "/repos/team/kits/tags?per_page=100&page=1",
            r#"[{"name":"v3","commit":{"sha":"abc123"}}]"#,
        ),
        (
            "/repos/team/kits/git/trees/abc123?recursive=1",
            r#"{"tree":[{"type":"blob","path":"workload/workload.yaml"}]}"#,
        ),
        (
            "/team/kits/abc123/workload/workload.yaml",
            "schemaVersion: '3'\nkind: workload\n",
        ),
    ]);
    let providers = ["broken/kits", "team/kits"].map(|repository| Provider {
        id: repository.into(),
        label: repository.into(),
        kind: "github".into(),
        repository: repository.into(),
        reference: String::new(),
        enabled: true,
    });
    let catalog = discover_with(&api, &providers, "").unwrap();
    assert_eq!(catalog.kits.len(), 1);
    assert_eq!(catalog.warnings.len(), 1);
    assert!(catalog.warnings[0].contains("broken/kits"));
    worker.join().unwrap();
}

#[test]
fn providers_round_trip_and_report_corruption() {
    let root = std::env::temp_dir().join(format!("sandy-catalog-{}", unique_id()));
    let providers = get_providers(&root).unwrap();
    save_providers(&root, providers.clone()).unwrap();
    assert_eq!(get_providers(&root).unwrap(), providers);
    std::fs::write(root.join("providers.json"), "not json").unwrap();
    assert!(get_providers(&root).unwrap_err().contains("providers.json"));
    std::fs::remove_dir_all(root).unwrap();
}

const REAL_V048_HELP_LINE: &str = "  Available agents: claude, codex, copilot, cursor, devin, docker-agent, droid, gemini, kiro, opencode, shell";

const ALL_DOCUMENTED: &[&str] = &[
    "claude",
    "codex",
    "copilot",
    "cursor",
    "devin",
    "docker-agent",
    "droid",
    "gemini",
    "kiro",
    "opencode",
    "shell",
];

#[test]
fn parse_available_agents_reads_the_real_help_line() {
    let agents =
        parse_available_agents(&format!("Description:\n  Run it.\n{REAL_V048_HELP_LINE}\n"));
    assert_eq!(agents, ALL_DOCUMENTED);
}

#[test]
fn parse_available_agents_is_case_and_decoration_tolerant() {
    // Reworded label casing, ANSI color codes around the values, trailing
    // backslashes from a Markdown hard-wrap, and heavy indentation.
    let help = "\x1b[1m  AVAILABLE AGENTS:\x1b[0m \x1b[36mclaude\x1b[0m, opencode, \\\n      shell";
    assert_eq!(
        parse_available_agents(help),
        ["claude", "opencode", "shell"]
    );
}

#[test]
fn parse_available_agents_rejects_garbage_and_missing_lines() {
    for help in [
        "",
        "Usage:\n  sbx run [AGENT]",
        "Available agents:",
        "Available agents: , ,, ",
        "Available agents: ./docs, --name, :4, not/valid",
    ] {
        assert!(
            parse_available_agents(help).is_empty(),
            "parsed {help:?} into agents"
        );
    }
}

#[test]
fn completion_output_beats_help_and_filters_protocol_noise() {
    // Real `sbx __complete run ""` output from an empty directory: the 11
    // agent names plus the directive trailer. The trailer must be filtered,
    // not turned into a phantom `builtin:4` kit.
    let completion = format!("{}\n:4\n", ALL_DOCUMENTED.join("\n"));
    let agents = builtin_agents(Some(&completion), None);
    assert_eq!(
        agents,
        BuiltinAgents {
            names: ALL_DOCUMENTED.iter().map(|name| (*name).into()).collect(),
            from_fallback: false,
        }
    );
    assert_eq!(agents.names, parse_available_agents(REAL_V048_HELP_LINE));
}

#[test]
fn empty_completion_falls_back_to_help_then_documented_list() {
    fn documented(names: &[String]) -> Vec<&str> {
        names.iter().map(String::as_str).collect()
    }
    // Unusable completion output, usable help: the help parse wins.
    let agents = builtin_agents(Some("./docs\n:4\n"), Some(REAL_V048_HELP_LINE));
    assert_eq!(
        agents,
        BuiltinAgents {
            names: ALL_DOCUMENTED.iter().map(|name| (*name).into()).collect(),
            from_fallback: false,
        }
    );
    // Nothing usable from the CLI at all: the documented list, flagged as
    // fallback so the caller can warn.
    let agents = builtin_agents(Some(""), Some("no agents here"));
    assert_eq!(documented(&agents.names), ALL_DOCUMENTED);
    assert!(agents.from_fallback);
    let agents = builtin_agents(None, None);
    assert_eq!(documented(&agents.names), ALL_DOCUMENTED);
    assert!(agents.from_fallback);
}

#[test]
fn documented_fallback_never_shadows_a_real_parse() {
    // A CLI that reports a subset must not be "corrected" to the full
    // documented list.
    let agents = builtin_agents(
        Some("claude\nshell\n:4\n"),
        Some("Available agents: claude, shell"),
    );
    assert_eq!(agents.names, ["claude", "shell"]);
    assert!(!agents.from_fallback);
}
