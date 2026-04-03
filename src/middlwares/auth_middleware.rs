use std::sync::Arc;
use axum::{extract::{Request, State}, middleware::Next, response::{IntoResponse, Response}, http::StatusCode};
use cookie::Cookie;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha384;
use tower_cookies::Cookies;
use std::collections::BTreeMap;

use crate::{AppState, rag_fn::generate_accesstoken::PayloadAccess};

pub async fn is_auth(State(state): State<Arc<AppState>>, cookies: Cookies, mut req: Request, next: Next) -> Response {
    let pool = &state.db_pool;
    if let Some(access_token)=cookies.get("access_token"){
        // verifying the access token
    
        let Ok(user) = parse_access_token(access_token, &state.jwt_secret) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response()
        };
        let Ok(_) = sqlx::query(r#"
        SELECT * FROM users WHERE id=$1 
        "#).bind(user.id).fetch_one(pool).await else {
            return StatusCode::UNAUTHORIZED.into_response()
        };


        req.extensions_mut().insert(user.clone());
        return next.run(req).await;
    };
    StatusCode::UNAUTHORIZED.into_response()


}   

pub fn parse_access_token(access_token:Cookie, secret_key: &str)->Result<PayloadAccess,StatusCode>{
    let Ok(key)=Hmac::new_from_slice(secret_key.as_bytes())else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let key:Hmac<Sha384>=key;
    let token_str=access_token.to_string();
    let token_vec:Vec<&str>=token_str.split("=").collect();
    let t=token_vec.get(1).ok_or(StatusCode::BAD_REQUEST)?;
    let Ok(claim)=t.verify_with_key(&key)else{
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let claim:BTreeMap<String,PayloadAccess>=claim;
    let Some(user)=claim.get("user") else{
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    Ok(user.clone())
}