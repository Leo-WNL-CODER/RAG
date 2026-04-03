use std::{collections::BTreeMap, sync::Arc};

use axum::{http::StatusCode, response::IntoResponse, extract::State};
use cookie::{Cookie, SameSite, time::{Duration, OffsetDateTime}};
use hmac::{Hmac, Mac};
use redis::Commands;
use sha2::Sha384;
use tower_cookies::Cookies;
use uuid::Uuid;
use jwt::VerifyWithKey;
use sqlx::Row;
use chrono::{DateTime, Utc};


use crate::{AppState, rag_fn::generate_accesstoken::{ PayloadAccess, access_token}, routes::signin::User};

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    cookies: Cookies
)->impl IntoResponse{

    let Ok(mut redis) = state.redis_client.get_connection() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };

    let pool = &state.db_pool;

    let Some(refresh_token)=cookies.get("refresh_token") else {
        return StatusCode::NOT_FOUND.into_response()
    };
    let secret = &state.jwt_secret;
    let Ok(key):Result<Hmac<Sha384>,_>=Hmac::new_from_slice(secret.as_bytes()) else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };

    let t_str=refresh_token.to_string();
    let t_v:Vec<&str>=t_str.split("=").collect();
    let Some(t)=t_v.get(1) else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };
    let Ok(claim):Result<BTreeMap<String, Uuid>,_>=t.verify_with_key(&key) else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };
    let Some(user_id)=claim.get("user_id") else{
        return StatusCode::NOT_FOUND.into_response()
    }; 

    let Ok(redis_token)=redis.get::<_,String>
    (format!("{}:{}","refresh_cookie",user_id)) else {
        return StatusCode::NOT_FOUND.into_response()
    };
    if (*t).to_string()!=redis_token{
        return StatusCode::NOT_FOUND.into_response()
    }

    //if redistoken and refresh_token matches:
    //fetch the user details get access token and store it as a cookie
    let Ok(row)=sqlx::query(r#"
    Select * from users where id=$1
    "#)
    .bind(user_id).fetch_one(pool).await else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let q=User{
        id:row.get("id"),
        email:row.get("email"),
        username:row.get("username"),
        password_hash:row.get("password_hash"),
        created_at:row.get::<DateTime<Utc>,_>("created_at"),
    };

    let user_payload=PayloadAccess{
        id:q.id,
        username:q.username,
        email:q.email
    };

    let Ok(access_token)=access_token(user_payload) else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };

    let mut access_cookie=Cookie::new("access_token", access_token);

    let access_expires=OffsetDateTime::now_utc()+Duration::minutes(15);
   
    access_cookie.set_http_only(true);
    access_cookie.set_expires(access_expires);
    access_cookie.set_same_site(SameSite::Lax);

    cookies.add(access_cookie);

    StatusCode::CREATED.into_response()
}