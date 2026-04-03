use chrono::Utc;
use ort::session::Session;
use redis::{Commands, Connection};
use similarity::{Similarity, similarity_traits::CosineSimilarity};
use tokenizers::Tokenizer;
use uuid::Uuid;

use crate::AppState;
use crate::routes::querying::{LatestChat, RedisChat, get_tokenized};
use crate::rag_fn::ask_model::{prompt_model};

pub async fn update_history(
    state: &std::sync::Arc<AppState>,
    redis_cli:&mut Connection,
    user_id:&Uuid,
    latest_conversation: LatestChat,
    old_summary: &str,
) -> Result<String,SummaryError> {

    // Combine latest interaction (important for accurate similarity)
    let interaction = format!(
        "User: {}\nAssistant: {}",
        latest_conversation.user_query,
        latest_conversation.llm_res
    );
    // Decide whether topic drift occurred
    let topic_shift = if old_summary.is_empty() {
        false
    } else {
        let interaction_clone = interaction.clone();
        let session_clone = state.session.clone();
        let tokenizer_clone = state.tokenizer.clone();
        let old_summary_clone = old_summary.to_string();
        
        let (interaction_vec, summary_vec) = match tokio::task::spawn_blocking(move || {
            let mut sess = session_clone.lock().unwrap();
            let i_vec = get_tokenized(&mut sess, &tokenizer_clone, &interaction_clone).ok()?;
            let s_vec = get_tokenized(&mut sess, &tokenizer_clone, &old_summary_clone).ok()?;
            Some((i_vec, s_vec))
        }).await {
            Ok(Some(v)) => v,
            _ => return Err(SummaryError::UnableToGenerateSummary),
        };

        match CosineSimilarity::similarity((&interaction_vec, &summary_vec)) {
            Some(sim) => sim < 0.7,
            None => true,
        }
    };

    if topic_shift && !old_summary.is_empty() {
        let db = &state.db_pool;

        let Ok(tokenized_old_summary)=state.tokenizer.encode(old_summary, true) else{
            return Err(SummaryError::UnableToParseModelResponse);
        };
        let token_count =tokenized_old_summary.get_ids().len();

        sqlx::query(
            r#"
            INSERT INTO conversation_summaries
                (user_id, summary, token_count, is_active, updated_at)
            VALUES ($1, $2, $3, false, $4)
            "#,
        )
        .bind(*user_id)
        .bind(old_summary)
        .bind(token_count as i32)
        .bind(Utc::now())
        .execute(db)
        .await
        .map_err(|_| SummaryError::DatabaseError)?;
    }
    // If topic drift → reset summary
    let base_summary = if topic_shift { "" } else { old_summary };


    // Prompt for updating summary
    let prompt = format!(
        r#"
System Instruction:
You are a conversation summarization engine.

Rules:
- Maintain a concise, factual running summary
- No greetings, filler, or repetition
- Merge if topic continues
- Add a new bullet if topic shifts
- If old summary is empty, create from scratch

----------------------------------------
Old Conversation Summary:
{}

----------------------------------------
Latest Interaction:
User:
{}

Assistant:
{}

----------------------------------------
Task:
Generate an UPDATED conversation summary.

Output:
- Bullet points only
- Minimal but complete
- No extra text
"#,
        base_summary,
        latest_conversation.user_query,
        latest_conversation.llm_res
    );

    // Call LLM to get updated summary
    let Ok(mut updated_summary) = prompt_model(&prompt).await else{
        return Err(SummaryError::UnableToGenerateSummary);
    };
    
    let Ok(tokenized_updated_summary)=state.tokenizer.encode(updated_summary.clone(), true) else{
        return Err(SummaryError::UnableToParseModelResponse);
    };
    
    let mut token_count=tokenized_updated_summary.get_ids().len();
    
    if token_count>400{

        let summary_prompt=summary_prompt(&updated_summary);
        
        if let Ok(new_summary)=prompt_model(&summary_prompt).await {
            updated_summary=new_summary;
        }else{
            return Err(SummaryError::UnableToGenerateSummary);
        };
        
        let Ok(tokenized_updated_summary)=state.tokenizer.encode(updated_summary.clone(), true) else{
            return Err(SummaryError::UnableToParseModelResponse);
        };
        token_count=tokenized_updated_summary.get_ids().len();
    }

    let redis_value = RedisChat {
        summary: updated_summary.clone(),
        token: token_count as i32,
        is_active: true,
    };

    let Ok(parsed_chat)=serde_json::to_string(&redis_value) else{
        return Err(SummaryError::UnableToParseModelResponse);
    };
    
    if redis_cli
    .set_ex::<_, _, ()>(format!("{}:{}","chat-summary",&user_id), parsed_chat, 7*24 * 60 * 60)
    .is_err(){
        return Err(SummaryError::DatabaseError);
    };
    Ok(updated_summary)
}

fn summary_prompt(summary:&str)->String{
    let prompt=format!(r#"System Instruction:
You are a long-term memory compression engine.

Your task is to compress an existing conversation summary
while preserving all important facts, decisions, constraints,
and ongoing tasks.

Rules:
- Do NOT add new information
- Do NOT remove important facts
- Merge related points
- Remove redundancy and verbose wording
- Keep factual, neutral tone
- Use bullet points only
- Keep it as short as possible while still complete

----------------------------------------
Existing Summary:
{}

----------------------------------------
Task:
Produce a COMPRESSED summary suitable for long-term memory.

Output:
- Bullet points only
- No headings
- No explanations
"#,summary);
prompt.to_string()
}


#[derive(Debug)]
pub enum SummaryError{
    UnableToConnectToModel,
    UnableToParseModelResponse,
    UnableToGenerateSummary,
    DatabaseError,
}