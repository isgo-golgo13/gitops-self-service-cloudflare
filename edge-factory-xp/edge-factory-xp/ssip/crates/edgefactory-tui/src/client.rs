use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::{json, Value};
use edgefactory_core::api::{AppRequest, AppStatus, Catalog, Receipt};
use edgefactory_core::{schema, xr};

/// What a submission produced: a file path (tofu) or a receipt (xp/demo).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum Outcome {
    File { written: PathBuf, next: String },
    Receipt(Receipt),
}

#[derive(Clone)]
pub enum Client {
    Http { base: String, http: reqwest::Client },
    Tofu { repo: PathBuf, zones: Vec<String> },
    Demo { polls: Arc<AtomicU32> },
}

impl Client {
    pub fn http(base: &str, token: &str) -> anyhow::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {token}"))?);
        Ok(Self::Http { base: base.trim_end_matches('/').to_string(), http: reqwest::Client::builder().default_headers(headers).build()? })
    }
    pub fn tofu(repo: &Path, zones: Vec<String>) -> anyhow::Result<Self> {
        anyhow::ensure!(repo.join("platform/spec.schema.json").exists(), "{} has no platform/spec.schema.json", repo.display());
        Ok(Self::Tofu { repo: repo.to_path_buf(), zones })
    }
    pub fn demo() -> Self {
        Self::Demo { polls: Arc::new(AtomicU32::new(0)) }
    }

    pub async fn catalog(&self) -> anyhow::Result<Catalog> {
        match self {
            Self::Http { base, http } => Ok(http.get(format!("{base}/api/v1/catalog")).send().await?.error_for_status()?.json().await?),
            Self::Tofu { repo, zones } => Ok(schema::catalog_from_files(&repo.join("platform/spec.schema.json"), zones.clone())?),
            Self::Demo { .. } => Ok(demo_catalog()),
        }
    }

    pub async fn submit(&self, req: &AppRequest, spec_schema: &Value) -> anyhow::Result<Outcome> {
        xr::validate(spec_schema, &req.spec)?;
        match self {
            Self::Http { base, http } => {
                let resp = http.post(format!("{base}/api/v1/requests")).json(req).send().await?;
                if !resp.status().is_success() {
                    let status = resp.status();
                    anyhow::bail!("{status}: {}", resp.text().await.unwrap_or_default());
                }
                Ok(Outcome::Receipt(resp.json().await?))
            }
            Self::Tofu { repo, .. } => {
                let dir = repo.join("infra/live/prod/requests");
                std::fs::create_dir_all(&dir)?;
                let path = dir.join(format!("{}.json", req.name));
                std::fs::write(&path, serde_json::to_string_pretty(&xr::request_file(req))? + "\n")?;
                Ok(Outcome::File { written: path, next: "git add, commit, open a PR (NGO) or `make push` (GO). The plan shows exactly what the platform will create.".into() })
            }
            Self::Demo { polls } => {
                polls.store(0, Ordering::Relaxed);
                Ok(Outcome::Receipt(Receipt {
                    namespace: req.namespace.clone(), name: req.name.clone(),
                    freight_digest: "sha256:4a1f9c1e000000000000000000000000000000000000000000000000d3m0d3m0".into(),
                    freight_reference: format!("ghcr.io/isgo-golgo13/edge-factory-xp/freight/{}:{}-demo", req.namespace, req.name),
                    requested_by: "demo@dronegrid.io".into(), applied_at: "2026-10-01T09:00:00Z".into(),
                }))
            }
        }
    }

    pub async fn status(&self, namespace: &str, name: &str, spec: &Value) -> anyhow::Result<AppStatus> {
        match self {
            Self::Http { base, http } => Ok(http.get(format!("{base}/api/v1/apps/{namespace}/{name}")).send().await?.error_for_status()?.json().await?),
            Self::Tofu { .. } => anyhow::bail!("status is reported by the OpenTofu plan/apply, not the terminal"),
            Self::Demo { polls } => {
                let n = polls.fetch_add(1, Ordering::Relaxed);
                let host = format!("{}.{}", spec["hostname"].as_str().unwrap_or("app"), spec["zone"].as_str().unwrap_or("dronegrid.io"));
                Ok(AppStatus {
                    namespace: namespace.into(), name: name.into(), ready: n >= 3, synced: true,
                    message: if n >= 3 { "Successfully composed".into() } else { "Composing resources".into() },
                    hostname: Some(host.clone()), worker: Some(name.into()),
                    d1_database_id: if spec["database"] == "d1" && n >= 1 { Some("5f3c…d1".into()) } else { None },
                    r2_bucket: if spec["objects"] == true { Some(format!("{name}-objects")) } else { None },
                    hyperdrive_id: if spec["database"] == "postgres" && n >= 1 { Some("8a2e…pg".into()) } else { None },
                    access_application_id: if spec["exposure"] == "private" && n >= 2 { Some("acc…".into()) } else { None },
                    load_balancer_id: if spec["exposure"] == "hybrid" && n >= 2 { Some("lb…".into()) } else { None },
                    wrangler_bindings: if n >= 3 { Some(json!({"name": name, "routes": [{"pattern": host, "custom_domain": true}]})) } else { None },
                })
            }
        }
    }
}

fn demo_catalog() -> Catalog {
    schema::catalog_from_files(Path::new("platform/spec.schema.json"), vec!["dronegrid.io".into(), "chargegun.io".into()])
        .unwrap_or_else(|_| Catalog {
            spec_schema: json!({"type":"object","required":["zone","hostname","tier","exposure"],"properties":{
                "zone":{"type":"string"},"hostname":{"type":"string","pattern":"^[a-z][a-z0-9-]{0,40}$"},
                "tier":{"type":"string","enum":["small","medium","large"]},"exposure":{"type":"string","enum":["public","private","hybrid"]},
                "database":{"type":"string","enum":["none","d1","postgres"]},"objects":{"type":"boolean"},"container":{"type":"boolean"},
                "origins":{"type":"array","items":{"type":"string"}}}}),
            zones: vec!["dronegrid.io".into(), "chargegun.io".into()], tiers: vec!["small".into(), "medium".into(), "large".into()],
        })
}
