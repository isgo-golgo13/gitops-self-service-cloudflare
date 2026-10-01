//! edgefactory-core: the pieces both the SSIP service and the TUI agree on.
//!
//! * [`schema`]  — discover the XRD's OpenAPI v3 schema and the platform catalog (EnvironmentConfigs)
//! * [`xr`]      — build and validate a `AppStack` composite resource from user answers
//! * [`freight`] — package a request as a signed OCI artifact in Harbor before it is applied
//! * [`api`]     — the wire types exchanged between TUI and service

pub mod api;
pub mod freight;
pub mod schema;
pub mod xr;

/// API group of the platform XRD.
pub const GROUP: &str = "edgefactory.dronegrid.io";
/// Served version of the platform XRD.
pub const VERSION: &str = "v1alpha1";
/// Composite kind app teams request.
pub const KIND: &str = "AppStack";
/// Plural resource name.
pub const PLURAL: &str = "appstacks";
/// Annotation carrying the freight digest (checked by the admission policy).
pub const ANNOTATION_FREIGHT: &str = "edgefactory.dronegrid.io/freight";
/// Annotation carrying the requester identity.
pub const ANNOTATION_REQUESTED_BY: &str = "edgefactory.dronegrid.io/requested-by";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("kubernetes: {0}")]
    Kube(#[from] kube::Error),
    #[error("xrd {0} has no served version {1}")]
    MissingVersion(String, String),
    #[error("request does not match the platform schema: {0}")]
    Schema(String),
    #[error("freight: {0}")]
    Freight(String),
    #[error("serialization: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
