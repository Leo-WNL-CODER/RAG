use std::sync::Arc;
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde_json::json;
use crate::AppState;

pub async fn check_health(State(state): State<Arc<AppState>>)->impl IntoResponse{
    
    let Ok(mut _redis_cli) = state.redis_client.get_connection() else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let Ok(_) = sqlx::query("SELECT 1").execute(&state.db_pool).await else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let Ok(_client)=state.client.health_check().await else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    (
        StatusCode::OK,
        Json(json!({
            "status":"ok"
        }))
    ).into_response()
}