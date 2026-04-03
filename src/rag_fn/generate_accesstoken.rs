use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey,Token, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::{Sha384};
use uuid::Uuid;
use std::{collections::BTreeMap, env};

#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct PayloadAccess{
    pub id:Uuid,
    pub username:String,
    pub email:String
}
pub fn access_token(payload:PayloadAccess)->Result<String,TokenError>{
    let Ok(secret_key)=env::var("JWT_ACCESS_TOKEN_SECRET") else{
        return Err(TokenError::FailedToGetSecretKey);
    };
    let Ok(key)=Hmac::new_from_slice(secret_key.as_bytes()) else{
        return Err(TokenError::FailedToGenerateToken);
    };

    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };

    let mut claims = BTreeMap::new();
    claims.insert("user", payload);

    let Ok(token)= Token::new(header, claims).sign_with_key(&(key as Hmac<Sha384>))else {
        return Err(TokenError::FailedToSignToken);
    }; 


    Ok(token.into())
}

#[derive(Debug)]
pub enum TokenError{
    FailedToGenerateToken,
    FailedToGetSecretKey,
    FailedToSignToken,
    FailedToVerifyToken,
    FailedToExtractClaims,
    FailedToParseClaims,
    FailedToGetToken,
}
