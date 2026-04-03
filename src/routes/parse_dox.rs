use std::{sync::Arc};

use axum::{extract::{Multipart, State}, response::IntoResponse};
use qdrant_client::{Payload, qdrant::{Condition, DeletePointsBuilder, Filter, PointStruct, UpsertPointsBuilder}};
use reqwest::{ multipart::{Form, Part}};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::json;
use tokio::task::spawn_blocking;
use tower_cookies::Cookies;
use uuid::Uuid;
use crate::{AppState, middlwares::auth_middleware::parse_access_token, rag_fn::chunking::chunk_text};

#[derive(Deserialize)]
pub struct PythonRes{
    text:Option<String>
}
pub async fn parse_doc(
    cookie:Cookies,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) ->impl IntoResponse {
    
    let client = &state.reqwest_client;
    let pg_pool = &state.db_pool;

    let field = match multipart.next_field().await {
        Ok(Some(f)) => f,
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };

    let file_name = field.file_name().unwrap_or("").to_string();

    let data = match field.bytes().await {
        Ok(d) => d,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    // ---- Send to Python parser ----
    let part = match Part::bytes(data.to_vec())
        .file_name(file_name.clone())
        .mime_str("application/octet-stream")
    {
        Ok(p) => p,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let parser_url = &state.parser_url;

    let res = match client
        .post(parser_url)
        .multipart(Form::new().part("file", part))
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let PythonRes { text } = match res.json::<PythonRes>().await {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let text = match text {
        Some(t) => t,
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if text.is_empty(){
        return StatusCode::BAD_REQUEST.into_response();
    }

    if text.len()>100_000{
        return StatusCode::BAD_REQUEST.into_response();
    }
    // ---- Chunking ----
    let enc = match state.tokenizer.encode(text.clone(), true) {
        Ok(e) => e,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };


    let chunks = match spawn_blocking({
        let state = state.clone();
        move || {
            let mut sess = state.session.lock().unwrap();
            chunk_text(enc, &mut sess) 
        }
    })
    .await
    {
        Ok(Ok(c)) => c,
        _=> return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };


    let Some(access_cookie)=(cookie.get("access_token")) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    
    
    let mut points: Vec<PointStruct>=vec![];
    
    let Ok(user) = parse_access_token(access_cookie, &state.jwt_secret) else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };
    
    let Ok(_)=sqlx::query("DELETE FROM doc_info where user_id=$1")
    .bind(&user.id)
    .execute(pg_pool)
    .await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };
    match state.client
    .delete_points( DeletePointsBuilder::new("test_collection")
    .points(Filter::must([Condition::matches(
        "user_id",user.id.to_string().clone(),
    )]))
    .wait(true),).await {
        Ok(r)=>{
            },
        Err(e)=>{
            
        }
    };

    let doc_id = Uuid::new_v4();


    let Ok(_) =  sqlx::query(
        "INSERT INTO doc_info (doc_id, user_id, file_name) VALUES ($1, $2, $3)",
    )
    .bind(&doc_id)
    .bind(&user.id)
    .bind(file_name.clone())
    .execute(pg_pool)
    .await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };

    for (_i,(v,span)) in chunks.iter().enumerate(){
        let point_id = Uuid::new_v4().to_string();
        let Ok(payload)=Payload::try_from(json!({
        "text":&text[span.0..span.1],
        "user_id":&user.id,
        "doc_id":&doc_id
        })) else{
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let p=PointStruct::new(
            point_id,
            v.clone(), 
            payload,
        );
        points.push(p);
    }
        
    let Ok(_insert_res) = state.client
    .upsert_points(UpsertPointsBuilder::new("test_collection", points).wait(true))
    .await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }; 
    
    (
        StatusCode::ACCEPTED,
        "Document uploaded and processing started",
    ).into_response()
    
}
