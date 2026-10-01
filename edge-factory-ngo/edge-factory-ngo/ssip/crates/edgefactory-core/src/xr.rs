//! Build, validate and project the request: as an AppStack XR (XP) or a requests/<name>.json (NGO/GO).
use serde_json::{json, Value};

use crate::api::{AppRequest, AppStatus};
use crate::{Error, Result, ANNOTATION_FREIGHT, ANNOTATION_REQUESTED_BY, GROUP, KIND, VERSION};

pub fn validate(spec_schema: &Value, spec: &Value) -> Result<()> {
    let validator = jsonschema::validator_for(spec_schema).map_err(|e| Error::Schema(format!("schema compile: {e}")))?;
    let errors: Vec<String> = validator.iter_errors(spec).map(|e| format!("{} at {}", e, e.instance_path)).collect();
    if !errors.is_empty() {
        return Err(Error::Schema(errors.join("; ")));
    }
    if spec["exposure"] == "hybrid" && spec["origins"].as_array().map(Vec::len).unwrap_or(0) == 0 {
        return Err(Error::Schema("exposure hybrid needs at least one external origin".into()));
    }
    Ok(())
}

/// XP: the composite resource manifest.
pub fn manifest(req: &AppRequest, freight_digest: &str, requested_by: &str) -> Value {
    json!({
        "apiVersion": format!("{GROUP}/{VERSION}"),
        "kind": KIND,
        "metadata": {
            "name": req.name, "namespace": req.namespace,
            "annotations": { ANNOTATION_FREIGHT: freight_digest, ANNOTATION_REQUESTED_BY: requested_by },
            "labels": { "app.kubernetes.io/managed-by": "edgefactory-ssip" }
        },
        "spec": req.spec
    })
}

/// NGO/GO: the file the OpenTofu root merges (requests/<name>.json).
pub fn request_file(req: &AppRequest) -> Value {
    json!({ "apps": { req.name.clone(): req.spec } })
}

pub fn canonical(req: &AppRequest, requested_by: &str) -> Result<Vec<u8>> {
    let mut m = manifest(req, "pending", requested_by);
    if let Some(a) = m["metadata"]["annotations"].as_object_mut() {
        a.remove(ANNOTATION_FREIGHT);
    }
    let canonical: serde_json::Map<String, Value> = serde_json::from_value(m)?;
    Ok(serde_json::to_vec(&canonical)?)
}

pub fn project_status(xr: &Value) -> AppStatus {
    let cond = |t: &str| -> (bool, String) {
        xr["status"]["conditions"].as_array().and_then(|cs| cs.iter().find(|c| c["type"] == t))
            .map(|c| (c["status"] == "True", c["message"].as_str().unwrap_or("").to_string())).unwrap_or((false, String::new()))
    };
    let (ready, msg_r) = cond("Ready");
    let (synced, msg_s) = cond("Synced");
    let s = &xr["status"];
    let str_of = |k: &str| s[k].as_str().map(str::to_string);
    AppStatus {
        namespace: xr["metadata"]["namespace"].as_str().unwrap_or("").into(),
        name: xr["metadata"]["name"].as_str().unwrap_or("").into(),
        ready, synced,
        message: if msg_s.is_empty() { msg_r } else { msg_s },
        hostname: str_of("hostname"), worker: str_of("worker"), d1_database_id: str_of("d1DatabaseId"),
        r2_bucket: str_of("r2Bucket"), hyperdrive_id: str_of("hyperdriveId"), access_application_id: str_of("accessApplicationId"),
        load_balancer_id: str_of("loadBalancerId"),
        wrangler_bindings: if s["wranglerBindings"].is_null() { None } else { Some(s["wranglerBindings"].clone()) },
    }
}
