//! OIDC bearer validation against the site SSO (JWKS discovery). The subject/email becomes the
//! requester identity recorded in the freight and on the XR.

use std::collections::HashMap;
use std::sync::RwLock;

use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Discovery {
    jwks_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,
}

pub struct Oidc {
    issuer: String,
    audience: String,
    jwks_uri: String,
    keys: RwLock<HashMap<String, DecodingKey>>,
}

impl Oidc {
    pub async fn discover(issuer: &str, audience: &str) -> anyhow::Result<Self> {
        let disc: Discovery = reqwest::get(format!("{}/.well-known/openid-configuration", issuer.trim_end_matches('/')))
            .await?
            .error_for_status()?
            .json()
            .await?;
        let me = Self {
            issuer: issuer.trim_end_matches('/').to_string(),
            audience: audience.to_string(),
            jwks_uri: disc.jwks_uri,
            keys: RwLock::new(HashMap::new()),
        };
        me.refresh().await?;
        Ok(me)
    }

    async fn refresh(&self) -> anyhow::Result<()> {
        let set: JwkSet = reqwest::get(&self.jwks_uri).await?.error_for_status()?.json().await?;
        let mut fresh = HashMap::new();
        for jwk in set.keys {
            if let (Some(kid), Ok(key)) = (jwk.common.key_id.clone(), DecodingKey::from_jwk(&jwk)) {
                fresh.insert(kid, key);
            }
        }
        if let Ok(mut keys) = self.keys.write() {
            *keys = fresh;
        }
        Ok(())
    }

    /// Returns the requester identity (email if present, else subject).
    pub async fn verify(&self, bearer: &str) -> anyhow::Result<String> {
        let header = decode_header(bearer)?;
        let kid = header.kid.ok_or_else(|| anyhow::anyhow!("token has no kid"))?;
        let key = match self.keys.read().ok().and_then(|k| k.get(&kid).cloned()) {
            Some(k) => k,
            None => {
                self.refresh().await?;
                self.keys
                    .read()
                    .ok()
                    .and_then(|k| k.get(&kid).cloned())
                    .ok_or_else(|| anyhow::anyhow!("unknown signing key {kid}"))?
            }
        };
        let mut validation = Validation::new(header.alg);
        if !matches!(header.alg, Algorithm::RS256 | Algorithm::ES256 | Algorithm::PS256) {
            anyhow::bail!("unsupported token algorithm");
        }
        validation.set_audience(&[self.audience.as_str()]);
        validation.set_issuer(&[self.issuer.as_str()]);
        let data = decode::<Claims>(bearer, &key, &validation)?;
        Ok(data.claims.email.unwrap_or(data.claims.sub))
    }
}
