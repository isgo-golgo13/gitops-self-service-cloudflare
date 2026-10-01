//! edgefactory-ssip: the only identity that creates AppStacks.
//!
//! Flow per request: OIDC bearer -> validate against the live XRD schema -> push signed
//! freight to Harbor -> server-side apply the XR with the freight digest -> return a receipt.
//! Status is read straight from the XR; the TUI polls it.

mod auth;
mod routes;
mod state;

use std::net::SocketAddr;

use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug, Clone)]
#[command(name = "edgefactory-ssip", version)]
pub struct Config {
    #[arg(long, env = "EDGEFACTORY_LISTEN", default_value = "0.0.0.0:8080")]
    pub listen: SocketAddr,
    #[arg(long, env = "EDGEFACTORY_FREIGHT_REPO")]
    pub freight_repo: String,
    #[arg(long, env = "REGISTRY_USER")]
    pub registry_user: String,
    #[arg(long, env = "REGISTRY_PASSWORD", hide_env_values = true)]
    pub registry_password: String,
    /// cosign key reference (hashivault://edgefactory). Unset = unsigned freight (dev only).
    #[arg(long, env = "COSIGN_KEY")]
    pub cosign_key: Option<String>,
    #[arg(long, env = "EDGEFACTORY_OIDC_ISSUER")]
    pub oidc_issuer: String,
    #[arg(long, env = "EDGEFACTORY_OIDC_AUDIENCE", default_value = "edgefactory")]
    pub oidc_audience: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let config = Config::parse();
    let state = state::AppState::new(config.clone()).await?;
    let app = routes::router(state);
    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    tracing::info!(listen = %config.listen, "edgefactory-ssip listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
