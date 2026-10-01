//! edgefactory: a ratatui self-service terminal for Cloudflare AppStacks.
//!
//! Three ways to run, one set of screens:
//!   --mode tofu  (NGO/GO)  validate against platform/spec.schema.json and write requests/<name>.json
//!   --mode xp    (XP)      submit to the SSIP service, which signs freight and applies the XR
//!   --demo                 built-in catalog, no files, no network: review the screens

mod client;
mod ui;

use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    Tofu,
    Xp,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "edgefactory", version, about = "Self-service Cloudflare app stacks")]
pub struct Args {
    /// tofu: write requests/<name>.json for OpenTofu (NGO/GO). xp: submit to the SSIP service.
    #[arg(long, value_enum, default_value = "tofu")]
    pub mode: Mode,
    /// Repository root (tofu mode): platform/spec.schema.json is read, requests/ is written.
    #[arg(long, env = "EDGEFACTORY_REPO", default_value = ".")]
    pub repo: PathBuf,
    /// Zones the platform offers (tofu mode); xp mode discovers them from the cluster.
    #[arg(long, env = "EDGEFACTORY_ZONES", value_delimiter = ',', default_value = "dronegrid.io")]
    pub zones: Vec<String>,
    /// SSIP service base URL (xp mode).
    #[arg(long, env = "EDGEFACTORY_SSIP", default_value = "https://edgefactory-ssip.edgefactory-system.svc.dronegrid.io")]
    pub ssip: String,
    /// OIDC bearer for the SSIP service (xp mode).
    #[arg(long, env = "EDGEFACTORY_TOKEN", hide_env_values = true, default_value = "")]
    pub token: String,
    /// Team namespace (xp mode) / request owner label (tofu mode).
    #[arg(long, env = "EDGEFACTORY_NAMESPACE", default_value = "team-platform")]
    pub namespace: String,
    /// Offline walkthrough with a built-in catalog.
    #[arg(long)]
    pub demo: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let api = if args.demo {
        client::Client::demo()
    } else {
        match args.mode {
            Mode::Tofu => client::Client::tofu(&args.repo, args.zones.clone())?,
            Mode::Xp => {
                anyhow::ensure!(!args.token.is_empty(), "set EDGEFACTORY_TOKEN (or run with --demo)");
                client::Client::http(&args.ssip, &args.token)?
            }
        }
    };
    let catalog = api.catalog().await?;
    if let Some(outcome) = ui::run(&args, catalog, api).await? {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    }
    Ok(())
}
