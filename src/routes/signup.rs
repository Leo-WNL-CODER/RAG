use argon2::{Argon2, password_hash::{SaltString, rand_core::OsRng, PasswordHasher}};
use axum::{Json, extract::State, http::StatusCode, response::{IntoResponse, Response}};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct SignUpPayload{
    pub username:String,
    pub email:String,
    pub password:String
}

pub async fn user_signup(
    State(state): State<Arc<AppState>>,
    Json(user): Json<SignUpPayload>
) -> Response {
    if user.email.is_empty() || user.password.is_empty() {
        return (StatusCode::BAD_REQUEST, "Invalid User Info...").into_response();
    }
    let pool = &state.db_pool;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let Ok(password_hash) = argon2.hash_password(user.password.as_bytes(), &salt) else {
        return (StatusCode::BAD_REQUEST, "Enter Password again").into_response();
    };

    let Ok(_) = sqlx::query(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        "#
    )
    .bind(&user.username)
    .bind(&user.email)
    .bind(password_hash.to_string())
    .execute(pool)
    .await
    else {
        return (StatusCode::BAD_REQUEST, "Failed to save the info").into_response();
    };

    (StatusCode::OK, "SignUp is successful").into_response()
}