#![allow(unused_imports)]
#![allow(warnings)]

use std::sync::Arc;
use std::str::FromStr;

use anyhow::Result;
use axum::{Router, extract::DefaultBodyLimit, http::HeaderValue, middleware, routing::{ get, post}};
use ort::session::Session;
use qdrant_client::Qdrant;
use reqwest::{Method, header::{AUTHORIZATION, CONTENT_TYPE}};
use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
use tokenizers::Tokenizer;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use axum_governor::GovernorLayer;
use lazy_limit::{init_rate_limiter, Duration, RuleConfig};
use crate::{db::qdrant_db::db_create_collection, middlwares::auth_middleware::{self, is_auth}, rag_fn::initialize::initialize, routes::{health::check_health, parse_dox::parse_doc, querying::user_querying, refetch_access_token::refresh, signin::user_signin, signup::{self, user_signup}, test_user_profile::user_profile}};
mod rag_fn;
mod routes;
mod db;
mod middlwares;

pub struct AppState{
    pub session: Arc<std::sync::Mutex<Session>>,
    pub tokenizer: Tokenizer,
    pub client: Qdrant,
    pub db_pool: sqlx::PgPool,
    pub redis_client: redis::Client,
    pub reqwest_client: reqwest::Client,
    pub jwt_secret: String,
    pub parser_url: String,
}


/// -----------------------------------------
/// MAIN
/// -----------------------------------------
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let connect_options = PgConnectOptions::from_str(&database_url)?
        .statement_cache_capacity(0); 
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    let redis_url = std::env::var("REDIS_URL")
        .expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url)?;

    let reqwest_client = reqwest::Client::new();
    let jwt_secret = std::env::var("JWT_ACCESS_TOKEN_SECRET")
        .expect("JWT_ACCESS_TOKEN_SECRET must be set");
    let parser_url = std::env::var("PYTHON_PARSER_URL")
        .unwrap_or_else(|_| "http://localhost:8000/parse".to_string());

    let Ok(ini)=initialize() else{
        return Err(anyhow::anyhow!("Failed to initialize model and tokenizer"))
    };
    
    let s_t: Arc<AppState> = Arc::new(AppState{
        session: Arc::new(std::sync::Mutex::new(ini.0)),
        tokenizer: ini.1,
        db_pool,
        redis_client,
        reqwest_client,
        jwt_secret,
        parser_url,
        client: {
            let Ok(client) = Qdrant::from_url(&std::env::var("QDRANT_URL").unwrap_or_default())
                .api_key(std::env::var("QDRANT_API_KEY").unwrap_or_default())
                .skip_compatibility_check()
                .build() else {
                    return Err(anyhow::anyhow!("Failed to initialize Qdrant client"))
                };
            client
        }    
    });

    let _create_collection = db_create_collection(&s_t.client).await;

    let frontend_url=std::env::var("FRONTEND_URL")
    .unwrap_or_else(|_| "http://localhost:5173".to_string());

    let cors = CorsLayer::new()
    .allow_origin(frontend_url.parse::<HeaderValue>()?)
    .allow_credentials(true)
    .allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ])
    .allow_headers([
        CONTENT_TYPE,
        AUTHORIZATION,
    ]);

    let rate_limit_layer = init_rate_limiter!(
        default: RuleConfig::new(Duration::seconds(1), 5), // 5 req/s globally
        routes: [
            ("/parseDoc", RuleConfig::new(Duration::seconds(60), 2)),
            ("/userQuery", RuleConfig::new(Duration::seconds(60), 6)),
            ("/refresh", RuleConfig::new(Duration::seconds(60), 10)),
            ("/signup", RuleConfig::new(Duration::seconds(60), 5)),
            ("/signin", RuleConfig::new(Duration::seconds(60), 5)),
        ]
    ).await;

    let protected=Router::new()
    .route("/parseDoc", post(parse_doc))
    .layer(DefaultBodyLimit::max(5*1024*1024))
    .route("/me",get(user_profile))
    .route("/userQuery", get(user_querying))
    .route_layer(middleware::from_fn_with_state(s_t.clone(), is_auth));

    let app=Router::new()
    .merge(protected)
    .route("/signup", post(user_signup))
    .route("/signin", post(user_signin))
    .route("/refresh", get(refresh))
    .route("/health", get(check_health))
    .layer(CookieManagerLayer::new())
    .layer(cors)
    .layer(rate_limit_layer)
    .with_state(s_t);

    
    let listner=tokio::net::TcpListener::bind("localhost:3001").await?;

    axum::serve(listner,app).await?;

    Ok(())
}
