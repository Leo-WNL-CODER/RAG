use axum::{http::StatusCode, Json, response::IntoResponse, extract::State};
use std::sync::Arc;
use cookie::{SameSite, time::OffsetDateTime};
use redis::Commands;
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies, cookie::time::Duration};
use uuid::Uuid;
use crate::{AppState, rag_fn::{generate_accesstoken::{PayloadAccess, access_token}, generate_refreshtoken::refresh_token}};
use argon2::{
    password_hash::{
        PasswordHash, PasswordVerifier, 
    },
    Argon2
};
use chrono::{DateTime,Utc};
use sqlx::Row;


#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct SignInPayload{
    pub email:String,
    pub password:String
}

#[derive(Debug)]
pub struct User{
    pub id:Uuid,
    pub username :String,
    pub email :String,
    pub password_hash:String,
    pub created_at:DateTime<Utc>
}

pub async fn user_signin(
    State(state): State<Arc<AppState>>,
    cookies: Cookies,
    Json(user_payload): Json<SignInPayload>,
)->impl IntoResponse{
    let pool = &state.db_pool;

    let Ok(mut redis) = state.redis_client.get_connection() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };

    let Ok(Some(row))=sqlx::query(r#"
    SELECT * FROM users WHERE email=$1
    "#).bind(user_payload.email)
   .fetch_optional(pool).await else{
        return  StatusCode::UNAUTHORIZED.into_response()
    };

    let q=User{
        id:row.get("id"),
        email:row.get("email"),
        username:row.get("username"),
        password_hash:row.get("password_hash"),
        created_at:row.get::<DateTime<Utc>,_>("created_at"),
    };
    
    let Ok(hash_password)=PasswordHash::new(&q.password_hash)else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let password_byte=user_payload.password.as_bytes();
    let Ok(_)=Argon2::default().verify_password(password_byte, &hash_password) else {
        return  StatusCode::UNAUTHORIZED.into_response()
    };

    
    //generate access token and refresh token and send them as cookies

    let Ok(access_token)=access_token(PayloadAccess{
        id:q.id,
        username:q.username,
        email:q.email
    }) else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };
    let mut access_cookie=Cookie::new("access_token", access_token);

    let access_expires=OffsetDateTime::now_utc()+Duration::hours(1);
   
    access_cookie.set_http_only(true);
    access_cookie.set_expires(access_expires);
    access_cookie.set_same_site(SameSite::None); // cross-site: Vercel → Render
    access_cookie.set_secure(true);              // required for SameSite::None
    access_cookie.set_path("/");
    //Refresh Token
    let Ok(refresh_token)=refresh_token(q.id) else{
        return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };
    let mut refresh_cookie=Cookie::new("refresh_token", refresh_token.clone());
    let refresh_expires=OffsetDateTime::now_utc()+Duration::days(3);

    refresh_cookie.set_http_only(true);
    refresh_cookie.set_same_site(SameSite::None); // cross-site: Vercel → Render
    refresh_cookie.set_secure(true);              // required for SameSite::None
    refresh_cookie.set_expires(refresh_expires);
    refresh_cookie.set_path("/refresh");          // scope refresh token to its route
    if redis
    .set_ex::<_, _, ()>(format!("{}:{}","refresh_cookie",q.id), refresh_token.clone(), 24 * 60 * 60)
    .is_err(){
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    cookies.add(access_cookie);
    cookies.add(refresh_cookie);
    (StatusCode::OK,"signin_successful").into_response()
}