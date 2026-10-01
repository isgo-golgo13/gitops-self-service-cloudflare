//! Discover the platform API: from the cluster (XP) or from platform/spec.schema.json (NGO/GO).
use std::path::Path;

use kube::api::{Api, DynamicObject, ListParams};
use kube::discovery::ApiResource;
use kube::Client;
use serde_json::Value;

use crate::api::Catalog;
use crate::{Error, Result, GROUP, VERSION};

const XRD_NAME: &str = "appstacks.edgefactory.dronegrid.io";
const LABEL_ZONE: &str = "edgefactory.dronegrid.io/zone";

fn xrd_resource() -> ApiResource {
    ApiResource {
        group: "apiextensions.crossplane.io".into(),
        version: "v2".into(),
        api_version: "apiextensions.crossplane.io/v2".into(),
        kind: "CompositeResourceDefinition".into(),
        plural: "compositeresourcedefinitions".into(),
    }
}

fn envconfig_resource() -> ApiResource {
    ApiResource {
        group: "apiextensions.crossplane.io".into(),
        version: "v1beta1".into(),
        api_version: "apiextensions.crossplane.io/v1beta1".into(),
        kind: "EnvironmentConfig".into(),
        plural: "environmentconfigs".into(),
    }
}

/// NGO/GO: the catalog is a file in the repository plus the zones the root knows about.
pub fn catalog_from_files(schema_path: &Path, zones: Vec<String>) -> Result<Catalog> {
    let spec_schema: Value = serde_json::from_str(&std::fs::read_to_string(schema_path).map_err(|e| Error::Schema(format!("{}: {e}", schema_path.display())))?)?;
    let tiers = spec_schema["properties"]["tier"]["enum"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default();
    Ok(Catalog { spec_schema, zones, tiers })
}

/// XP: the catalog comes from the XRD and the EnvironmentConfigs.
pub async fn catalog(client: &Client) -> Result<Catalog> {
    let api: Api<DynamicObject> = Api::all_with(client.clone(), &xrd_resource());
    let xrd = api.get(XRD_NAME).await?;
    let served = xrd.data["spec"]["versions"].as_array().cloned().unwrap_or_default().into_iter()
        .find(|v| v["name"] == VERSION && v["served"] == true)
        .ok_or_else(|| Error::MissingVersion(XRD_NAME.into(), VERSION.into()))?;
    let spec_schema = served["schema"]["openAPIV3Schema"]["properties"]["spec"].clone();
    let envs: Api<DynamicObject> = Api::all_with(client.clone(), &envconfig_resource());
    let items = envs.list(&ListParams::default()).await?.items;
    let mut zones: Vec<String> = items.iter().filter_map(|o| o.metadata.labels.as_ref()?.get(LABEL_ZONE).cloned()).collect();
    zones.sort();
    let tiers = items.iter().find(|o| o.metadata.name.as_deref() == Some("edgefactory-platform"))
        .and_then(|o| o.data["data"]["tiers"].as_object().map(|m| m.keys().cloned().collect()))
        .unwrap_or_else(|| vec!["small".into(), "medium".into(), "large".into()]);
    Ok(Catalog { spec_schema, zones, tiers })
}

pub fn xr_resource() -> ApiResource {
    ApiResource {
        group: GROUP.into(),
        version: VERSION.into(),
        api_version: format!("{GROUP}/{VERSION}"),
        kind: crate::KIND.into(),
        plural: crate::PLURAL.into(),
    }
}
