//! The HTTP layer that exposes the worker to the outside world (n8n).
//!
//! One route: POST /run/<endpoint_id>. The JSON body becomes the automation's
//! args; the response is a JSON envelope with the run report and the output.
//!
//! Bound to 127.0.0.1 only for now: n8n runs on the same machine, so the
//! endpoint is NOT reachable from the network. Opening it up is a later, explicit
//! decision (auth, etc.) — localhost keeps it safe by default.

use std::collections::HashMap;
use std::net::SocketAddr;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::post,
};
use serde_json::json;

use crate::engine::worker::{WorkerHandle, start_worker};
use crate::models::engine_error::EngineErrorKind;

/// Build the router, wiring the shared WorkerHandle into every request.
fn app_router(handle: WorkerHandle) -> Router {
    Router::new()
        .route("/run/:endpoint_id", post(run_handler))
        .with_state(handle)
}

/// POST /run/<endpoint_id> with a flat JSON object body of string args.
/// The body is optional (an automation may take no inputs).
async fn run_handler(
    Path(endpoint_id): Path<String>,
    State(handle): State<WorkerHandle>,
    body: Option<Json<HashMap<String, String>>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let args = body.map(|Json(a)| a).unwrap_or_default();

    match handle.run(endpoint_id, args).await {
        Ok(report) => {
            // Whole report goes back so the caller sees the run log AND output.
            let payload = json!({
                "ok": report.succeeded(),
                "outcome": report.outcome,
                "output": report.output,
                "results": report.results,
            });
            (StatusCode::OK, Json(payload))
        }
        Err(e) => {
            // Map the engine error to a sensible HTTP status.
            let status = match &e {
                EngineErrorKind::EndpointNotFound { .. } => StatusCode::NOT_FOUND,
                EngineErrorKind::MissingRequiredInput { .. } => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(json!({ "ok": false, "error": e.to_string() })))
        }
    }
}

/// Start the worker thread AND the HTTP server, listening on 127.0.0.1:<port>.
/// Runs until the process exits. Async because axum is async; the blocking GUI
/// work happens on the worker's own OS thread, not here.
pub async fn serve(port: u16) -> Result<(), EngineErrorKind> {
    let handle = start_worker();
    let router = app_router(handle);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| EngineErrorKind::ServerBind {
            addr: addr.to_string(),
            source,
        })?;

    println!("worker server listening on http://{addr}");
    println!("  POST /run/<endpoint_id>  with a JSON body of args");

    axum::serve(listener, router)
        .await
        .map_err(|source| EngineErrorKind::ServerBind {
            addr: addr.to_string(),
            source,
        })
}

/// Bind the server, then run its accept loop in a background task and return.
/// Used by the desktop app: it `await`s this so a port conflict surfaces
/// immediately, then the server keeps running for the app's lifetime.
pub async fn serve_in_background(port: u16) -> Result<(), EngineErrorKind> {
    let handle = start_worker();
    let router = app_router(handle);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| EngineErrorKind::ServerBind {
            addr: addr.to_string(),
            source,
        })?;

    // Spawn the accept loop; binding already succeeded above.
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("worker server stopped: {e}");
        }
    });

    Ok(())
}
