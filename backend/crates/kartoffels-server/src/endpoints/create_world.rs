use crate::error::{AppError, AppResult};
use crate::AppState;
use anyhow::Context;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use kartoffels::{World, WorldConfig, WorldId};
use serde::Serialize;
use std::sync::Arc;
use tokio::fs::File;
use tokio::sync::RwLock;
use tracing::info;

// TODO must be password-protected
pub async fn handle(
    State(state): State<Arc<RwLock<AppState>>>,
    request: Json<WorldConfig>,
) -> AppResult<impl IntoResponse> {
    let mut state = state.write().await;
    let id = WorldId::new(&mut rand::thread_rng());
    let config = request.0;

    info!(?id, ?config, "creating new world");

    if state.has_world_named(&config.name) {
        return Err(AppError::Other(
            StatusCode::BAD_REQUEST,
            "world with this name already exists".into(),
        ));
    }

    let file = if let Some(data) = &state.data {
        let file = data.join(id.to_string()).with_extension("world");

        let file = File::create(&file)
            .await
            .context("couldn't create world's file")
            .map_err(AppError::MAP_HTTP_500)?;

        Some(file.into_std().await)
    } else {
        None
    };

    let world =
        World::create(id, config, file).map_err(AppError::MAP_HTTP_400)?;

    state.worlds.insert(id, world);

    Ok(Json(Response { id }))
}

#[derive(Clone, Debug, Serialize)]
struct Response {
    id: WorldId,
}
