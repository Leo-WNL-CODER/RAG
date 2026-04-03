use hmac::{Hmac, Mac};
use jwt::{Header, SignWithKey, Token};
use sha2::{Sha384};
use uuid::Uuid;
use std::{collections::BTreeMap, env};
use crate::rag_fn::generate_accesstoken::TokenError;

pub fn refresh_token(user_id:Uuid)->Result<String,TokenError>{
    dotenvy::dotenv().ok();
    let Ok(secret)=env::var("JWT_REFRESH_TOKEN_SECRET") else{
        return Err(TokenError::FailedToGetSecretKey)
    };
    let Ok(key)=Hmac::new_from_slice(secret.as_bytes()) else{
        return Err(TokenError::FailedToGenerateToken)
    };

    let header=Header{
        algorithm:jwt::AlgorithmType::Hs384,
        ..Default::default()
    };

    let mut claims: BTreeMap<&str, Uuid>=BTreeMap::new();
    claims.insert("user_id", user_id);
    let Ok(token)=Token::new(header, claims).sign_with_key(&(key as Hmac<Sha384>)) else{
        return Err(TokenError::FailedToSignToken)
    };
    Ok(token.into())
}