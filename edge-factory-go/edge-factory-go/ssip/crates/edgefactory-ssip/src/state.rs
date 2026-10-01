use std::sync::Arc;

use kube::Client;
use edgefactory_core::freight::FreightStore;

use crate::auth::Oidc;
use crate::Config;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub freight: Arc<FreightStore>,
    pub oidc: Arc<Oidc>,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let client = Client::try_default().await?;
        let freight = FreightStore {
            repository: config.freight_repo,
            username: config.registry_user,
            password: config.registry_password,
            cosign_key: config.cosign_key,
        };
        let oidc = Oidc::discover(&config.oidc_issuer, &config.oidc_audience).await?;
        Ok(Self { client, freight: Arc::new(freight), oidc: Arc::new(oidc) })
    }
}
