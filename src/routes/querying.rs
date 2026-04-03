use std::sync::{Arc};

use axum::extract::{Query, State};
use chrono::{Utc};
use ndarray::{ArrayViewD, Axis, Ix3};
use ort::{session::Session, value::Tensor};
use qdrant_client::qdrant::{Condition, Filter, QueryPointsBuilder};
use redis::Commands;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tokenizers::{Tokenizer};
use tower_cookies::Cookies;
use sqlx::Row;

use crate::{AppState, middlwares::auth_middleware::parse_access_token, rag_fn::{ask_model::prompt_model, get_final_prompt::get_prompt, mean_pooling::mean_pooling, summary_provider::update_history}};


#[derive(Deserialize,Clone,Debug)]
pub struct SearchParams {
   pub q: String,
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct LatestChat{
    pub user_query:String,
    pub llm_res:String
}

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct RedisChat{
    pub token:i32,
    pub summary:String,
    pub is_active:bool,
}

pub async fn user_querying(
    cookie:Cookies,
    State(state):State<Arc<AppState>>,
    Query(user_query):Query<SearchParams>
)->Result<String,StatusCode>
{
    
    let Ok(mut redis_client) = state.redis_client.get_connection() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };
    let db_client = &state.db_pool;

    //getting chat history-----
    if let Some(access_cookie)=cookie.get("access_token"){
        let Ok(user) = parse_access_token(access_cookie, &state.jwt_secret) else{
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        };

        let state_clone = state.clone();
        let user_query_clone = user_query.q.clone();
        let pooled = match tokio::task::spawn_blocking(move || {
            let mut session = state_clone.session.lock().unwrap();
            get_tokenized(&mut session, &state_clone.tokenizer, &user_query_clone)
        }).await {
            Ok(Ok(p)) => p,
            _ => {
                return Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        };
        let search_result= match state.client
        .query(
            QueryPointsBuilder::new("test_collection")
                .query(pooled).limit(8).with_payload(true).filter(
                    Filter::must(
                        [Condition
                        ::matches("user_id", user.id.to_string())]
                    )
                )
        )
        .await {
            Ok(r)=>r,
            Err(e)=>{
                return Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        };

        let redis_chat_sum: Result<String, redis::RedisError>=redis_client.get(format!("{}:{}","chat-summary",&user.id));
        
        let history=match redis_chat_sum{
            Ok(chat)=>{
                let parsed_redis_sum: Result<RedisChat,serde_json::Error>=serde_json::from_str(&chat);
                let Ok(new_chat)=parsed_redis_sum else{
                    return Err(StatusCode::INTERNAL_SERVER_ERROR)
                };
                new_chat.summary

            },
            Err(_)=>{
                //making the db call to fetch the active summary
                let row=sqlx::query("Select * from conversation_summaries where user_id=$1 and is_active=$2")
                .bind(user.id)
                .bind(true)
                .fetch_one(db_client).await;

                let (t_c,summary)=if row.is_err(){
                    (0,"".into())
                }else{
                    let Ok(r)=row else{
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    };
                    let text:String=r.get("summary");
                    let t:i32=r.get("token_count");
                    (t,text)
                };
                let chat=RedisChat{
                    token:t_c,
                    summary:summary.clone(),
                    is_active:true,
                };
                let Ok(parsed_chat)=serde_json::to_string(&chat) else{
                    return Err(StatusCode::INTERNAL_SERVER_ERROR)
                };
                if redis_client
                .set_ex::<_, _, ()>(format!("{}:{}","chat-summary",&user.id), parsed_chat, 7*24 * 60 * 60)
                .is_err(){
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                };

                summary
            }
        };

        let final_prompt=get_prompt(&history,&search_result.result,&user_query.q);

        let Ok(model_res)=prompt_model(&final_prompt).await else{
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        };

        if update_history(
        &state,
        &mut redis_client,
        &user.id,
        LatestChat{ 
        user_query: user_query.q, 
        llm_res: model_res.clone() },
        &history).await.is_err(){
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        };
        
        return Ok(model_res)
        
    };

    Err(StatusCode::NOT_FOUND)
}

pub fn get_tokenized( session:&mut Session,
    tokenizer:&Tokenizer,
    user_query:&str
)->Result<Vec<f32>,StatusCode>{
    let Ok(enc)=tokenizer.encode(user_query, true) else{
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };
    let input_ids: Vec<i64> = enc.get_ids().iter().map(|x| *x as i64).collect();
    let attention_mask: Vec<i64> = enc.get_attention_mask().iter().map(|x| *x as i64).collect();
    let token_type_ids = vec![0i64; input_ids.len()];
    let shape = [1, input_ids.len()];

    let Ok(ids_val) = Tensor::from_array((shape, input_ids)) else{
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };
    let Ok(mask_val) = Tensor::from_array((shape, attention_mask.clone())) else{
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };
    let Ok(types_val) = Tensor::from_array((shape, token_type_ids)) else{
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };

    let Ok(outputs) = session
    .run(ort::inputs![
        "input_ids" => ids_val,
        "attention_mask" => mask_val,
        "token_type_ids" => types_val
    ]) else{
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };
    
    let Ok(emb_3d): Result<ArrayViewD<f32>,_> = outputs[0].try_extract_array() else{
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };
    let Ok(emb_3d)= emb_3d.into_dimensionality::<Ix3>() else{
        return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };
    let emb_2d = emb_3d.index_axis(Axis(0), 0).to_owned();

    let pooled = mean_pooling(&emb_2d, &attention_mask);
    Ok(pooled)
}