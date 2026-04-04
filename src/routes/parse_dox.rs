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
        _ => { eprintln!("[parse_doc] ERR: multipart field missing"); return StatusCode::BAD_REQUEST.into_response() },
    };

    let file_name = field.file_name().unwrap_or("").to_string();
    eprintln!("[parse_doc] file_name={}", file_name);

    let data = match field.bytes().await {
        Ok(d) => d,
        Err(e) => { eprintln!("[parse_doc] ERR: bytes read failed: {}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response() },
    };

    let part = match Part::bytes(data.to_vec())
        .file_name(file_name.clone())
        .mime_str("application/octet-stream")
    {
        Ok(p) => p,
        Err(e) => { eprintln!("[parse_doc] ERR: mime_str failed: {}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response() },
    };

    let parser_url = &state.parser_url;
    eprintln!("[parse_doc] calling parser at {}", parser_url);

    let res = match client
        .post(parser_url)
        .multipart(Form::new().part("file", part))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => { eprintln!("[parse_doc] ERR: parser request failed: {}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response() },
    };

    eprintln!("[parse_doc] parser status={}", res.status());

    if !res.status().is_success() {
        eprintln!("[parse_doc] ERR: parser returned non-2xx: {}", res.status());
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let PythonRes { text } = match res.json::<PythonRes>().await {
        Ok(v) => v,
        Err(e) => { eprintln!("[parse_doc] ERR: json parse failed: {}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response() },
    };

    let text = match text {
        Some(t) => t,
        None => { eprintln!("[parse_doc] ERR: text is None"); return StatusCode::INTERNAL_SERVER_ERROR.into_response() },
    };

    eprintln!("[parse_doc] text_len={}", text.len());

    if text.is_empty(){
        return StatusCode::BAD_REQUEST.into_response();
    }

    if text.len()>100_000{
        return StatusCode::BAD_REQUEST.into_response();
    }

    let enc = match state.tokenizer.encode(text.clone(), true) {
        Ok(e) => e,
        Err(e) => { eprintln!("[parse_doc] ERR: tokenizer failed: {}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response() },
    };

    eprintln!("[parse_doc] tokenized, starting chunking");

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
        e => { eprintln!("[parse_doc] ERR: chunking failed: {:?}", e); return StatusCode::INTERNAL_SERVER_ERROR.into_response() },
    };

    eprintln!("[parse_doc] chunks={}", chunks.len());

    let Some(access_cookie)=(cookie.get("access_token")) else {
        eprintln!("[parse_doc] ERR: no access_token cookie");
        return StatusCode::NOT_FOUND.into_response();
    };
    
    let mut points: Vec<PointStruct>=vec![];
    
    let Ok(user) = parse_access_token(access_cookie, &state.jwt_secret) else{
        eprintln!("[parse_doc] ERR: token parse failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };

    eprintln!("[parse_doc] user_id={}", user.id);
    
    let Ok(_)=sqlx::query("DELETE FROM doc_info where user_id=$1")
    .bind(&user.id)
    .execute(pg_pool)
    .await else {
        eprintln!("[parse_doc] ERR: DELETE doc_info failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };

    eprintln!("[parse_doc] DELETE ok, deleting qdrant points");

    match state.client
    .delete_points( DeletePointsBuilder::new("test_collection")
    .points(Filter::must([Condition::matches(
        "user_id",user.id.to_string().clone(),
    )]))
    .wait(true),).await {
        Ok(_)=> eprintln!("[parse_doc] qdrant delete ok"),
        Err(e)=> eprintln!("[parse_doc] WARN: qdrant delete failed: {}", e),
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
        eprintln!("[parse_doc] ERR: INSERT doc_info failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };

    eprintln!("[parse_doc] INSERT doc_info ok, building qdrant points");

    for (_i,(v,span)) in chunks.iter().enumerate(){
        let point_id = Uuid::new_v4().to_string();
        let Ok(payload)=Payload::try_from(json!({
        "text":&text[span.0..span.1],
        "user_id":&user.id,
        "doc_id":&doc_id
        })) else{
            eprintln!("[parse_doc] ERR: payload build failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let p=PointStruct::new(
            point_id,
            v.clone(), 
            payload,
        );
        points.push(p);
    }

    eprintln!("[parse_doc] upserting {} points to qdrant", points.len());
        
    let Ok(_insert_res) = state.client
    .upsert_points(UpsertPointsBuilder::new("test_collection", points).wait(true))
    .await else {
        eprintln!("[parse_doc] ERR: qdrant upsert failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }; 

    eprintln!("[parse_doc] SUCCESS");
    
    (
        StatusCode::ACCEPTED,
        "Document uploaded and processing started",
    ).into_response()
    
}
