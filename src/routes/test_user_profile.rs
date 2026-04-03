use axum::{Extension, Json};

use crate::rag_fn::generate_accesstoken::PayloadAccess;

pub async fn user_profile(Extension(user):Extension<PayloadAccess>)->Json<PayloadAccess>{
    Json(user)
}