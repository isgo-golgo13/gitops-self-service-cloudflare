use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use kube::api::{Api, DynamicObject, Patch, PatchParams};
use serde_json::{json, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tower_http::trace::TraceLayer;
use edgefactory_core::api::{AppRequest, AppStatus, Catalog, Receipt};
use edgefactory_core::freight::sha256_hex;
use edgefactory_core::{schema, xr};

use crate::state::AppState;

type ApiError = (StatusCode, Json<Value>);

fn err(status: StatusCode, msg: impl ToString) -> ApiError {
    (status, Json(json!({ "error": msg.to_string() })))
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/v1/catalog", get(catalog))
        .route("/api/v1/requests", post(create))
        .route("/api/v1/apps/{namespace}/{name}", get(status))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn requester(state: &AppState, headers: &HeaderMap) -> Result<String, ApiError> {
    let bearer = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "missing bearer token"))?;
    state
        .oidc
        .verify(bearer)
        .await
        .map_err(|e| err(StatusCode::UNAUTHORIZED, e))
}

async fn catalog(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Catalog>, ApiError> {
    requester(&state, &headers).await?;
    let catalog = schema::catalog(&state.client)
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(catalog))
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AppRequest>,
) -> Result<(StatusCode, Json<Receipt>), ApiError> {
    let requested_by = requester(&state, &headers).await?;
    let spec_schema = schema::catalog(&state.client)
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, e))?
        .spec_schema;
    xr::validate(&spec_schema, &req.spec).map_err(|e| err(StatusCode::UNPROCESSABLE_ENTITY, e))?;

    // 1. Freight first: the request is durable and signed before anything touches the cluster.
    let canonical = xr::canonical(&req, &requested_by).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let request_yaml = serde_yaml::to_string(&serde_json::from_slice::<Value>(&canonical).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let schema_digest = sha256_hex(&serde_json::to_vec(&spec_schema).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?);
    let stored = state
        .freight
        .push(&req.namespace, &req.name, &requested_by, &schema_digest, request_yaml.as_bytes())
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, e))?;

    // 2. Apply the XR carrying the freight digest (admission policy checks identity + digest).
    let manifest = xr::manifest(&req, &stored.digest, &requested_by);
    let obj: DynamicObject = serde_json::from_value(manifest).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let api: Api<DynamicObject> = Api::namespaced_with(state.client.clone(), &req.namespace, &schema::xr_resource());
    api.patch(&req.name, &PatchParams::apply("edgefactory-ssip").force(), &Patch::Apply(&obj))
        .await
        .map_err(|e| err(StatusCode::BAD_GATEWAY, e))?;

    let applied_at = OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_default();
    tracing::info!(namespace = %req.namespace, name = %req.name, freight = %stored.digest, requested_by = %requested_by, "applied");
    Ok((
        StatusCode::ACCEPTED,
        Json(Receipt {
            namespace: req.namespace,
            name: req.name,
            freight_digest: stored.digest,
            freight_reference: stored.reference,
            requested_by,
            applied_at,
        }),
    ))
}

async fn status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<AppStatus>, ApiError> {
    requester(&state, &headers).await?;
    let api: Api<DynamicObject> = Api::namespaced_with(state.client.clone(), &namespace, &schema::xr_resource());
    let obj = api.get(&name).await.map_err(|e| err(StatusCode::NOT_FOUND, e))?;
    let value = serde_json::to_value(&obj).map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(xr::project_status(&value)))
}
