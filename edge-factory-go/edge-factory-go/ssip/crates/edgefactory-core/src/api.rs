//! Wire types shared by the service and the terminal. Provider-free by design.
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// What the terminal renders prompts from: the spec schema plus the platform catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub spec_schema: Value,
    pub zones: Vec<String>,
    pub tiers: Vec<String>,
}

/// A request as the app team expresses it. `spec` follows platform/spec.schema.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRequest {
    pub namespace: String,
    pub name: String,
    pub spec: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub namespace: String,
    pub name: String,
    pub freight_digest: String,
    pub freight_reference: String,
    pub requested_by: String,
    pub applied_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppStatus {
    pub namespace: String,
    pub name: String,
    pub ready: bool,
    pub synced: bool,
    pub message: String,
    pub hostname: Option<String>,
    pub worker: Option<String>,
    pub d1_database_id: Option<String>,
    pub r2_bucket: Option<String>,
    pub hyperdrive_id: Option<String>,
    pub access_application_id: Option<String>,
    pub load_balancer_id: Option<String>,
    pub wrangler_bindings: Option<Value>,
}
