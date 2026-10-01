//! Freight: every request becomes an immutable, signed OCI artifact in Harbor *before* it is
//! applied. The artifact's digest travels on the XR as an annotation; the admission policy
//! refuses XRs without one. Harbor is therefore the audit ledger of every request ever made.
//!
//! Layout of one freight artifact (ORAS-style):
//!   config   application/vnd.cencera.edgefactory.freight.config.v1+json  { requester, timestamp, schema digest }
//!   layer 0  application/vnd.cencera.edgefactory.request.v1+yaml         the XR manifest as submitted
//! Signature: `cosign sign --key <kms uri>` against the pushed digest (OpenBao Transit key).

use oci_client::client::{ClientConfig, ClientProtocol, Config, ImageLayer};
use oci_client::manifest::OciImageManifest;
use oci_client::secrets::RegistryAuth;
use oci_client::{Client, Reference};
use serde::Serialize;
use sha2::{Digest, Sha256};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::{Error, Result};

pub const MEDIA_CONFIG: &str = "application/vnd.cencera.edgefactory.freight.config.v1+json";
pub const MEDIA_REQUEST: &str = "application/vnd.cencera.edgefactory.request.v1+yaml";

#[derive(Debug, Clone)]
pub struct FreightStore {
    /// e.g. ghcr.io/isgo-golgo13/edge-factory-xp/freight
    pub repository: String,
    pub username: String,
    pub password: String,
    /// cosign key reference, e.g. hashivault://edgefactory (OpenBao Transit). None = unsigned (dev only).
    pub cosign_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct FreightConfig<'a> {
    schema: u32,
    requested_by: &'a str,
    namespace: &'a str,
    name: &'a str,
    requested_at: String,
    spec_schema_sha256: &'a str,
    request_sha256: String,
}

#[derive(Debug, Clone)]
pub struct Stored {
    pub reference: String,
    pub digest: String,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

impl FreightStore {
    /// Push the request as an OCI artifact tagged `<namespace>-<name>-<unix seconds>` and return
    /// its manifest digest. The tag is for humans; the digest is what the cluster carries.
    pub async fn push(
        &self,
        namespace: &str,
        name: &str,
        requested_by: &str,
        spec_schema_sha256: &str,
        request_yaml: &[u8],
    ) -> Result<Stored> {
        let now = OffsetDateTime::now_utc();
        let tag = format!("{namespace}-{name}-{}", now.unix_timestamp());
        let reference: Reference = format!("{}/{}:{}", self.repository, namespace, tag)
            .parse()
            .map_err(|e| Error::Freight(format!("bad reference: {e}")))?;
        let config = FreightConfig {
            schema: 1,
            requested_by,
            namespace,
            name,
            requested_at: now.format(&Rfc3339).map_err(|e| Error::Freight(e.to_string()))?,
            spec_schema_sha256,
            request_sha256: sha256_hex(request_yaml),
        };
        let config_bytes = serde_json::to_vec_pretty(&config)?;
        let client = Client::new(ClientConfig {
            protocol: ClientProtocol::Https,
            ..ClientConfig::default()
        });
        let auth = RegistryAuth::Basic(self.username.clone(), self.password.clone());
        let layers = vec![ImageLayer::new(request_yaml.to_vec(), MEDIA_REQUEST.into(), None)];
        let config = Config::new(config_bytes, MEDIA_CONFIG.into(), None);
        let mut manifest = OciImageManifest::build(&layers, &config, None);
        manifest.artifact_type = Some(MEDIA_REQUEST.into());
        let response = client
            .push(&reference, &layers, config, &auth, Some(manifest))
            .await
            .map_err(|e| Error::Freight(format!("push: {e}")))?;
        let digest = response
            .manifest_url
            .rsplit('/')
            .next()
            .filter(|d| d.starts_with("sha256:"))
            .map(str::to_string)
            .ok_or_else(|| Error::Freight("registry returned no manifest digest".into()))?;
        let stored = Stored { reference: reference.whole(), digest };
        if let Some(key) = &self.cosign_key {
            self.sign(&stored, key).await?;
        }
        Ok(stored)
    }

    /// `cosign sign --key <kms> <repo>@<digest>`; credentials via COSIGN_* / VAULT_* env.
    async fn sign(&self, stored: &Stored, key: &str) -> Result<()> {
        let repo = stored
            .reference
            .split(':')
            .next()
            .unwrap_or(&stored.reference)
            .to_string();
        let target = format!("{repo}@{}", stored.digest);
        let status = tokio::process::Command::new("cosign")
            .args(["sign", "--yes", "--tlog-upload=false", "--key", key, &target])
            .env("COSIGN_PASSWORD", "")
            .env("COSIGN_REGISTRY_USERNAME", &self.username)
            .env("COSIGN_REGISTRY_PASSWORD", &self.password)
            .status()
            .await
            .map_err(|e| Error::Freight(format!("cosign not runnable: {e}")))?;
        if !status.success() {
            return Err(Error::Freight(format!("cosign sign failed with {status}")));
        }
        Ok(())
    }
}
