use std::env;
use groq_api_rust::{AsyncGroqClient, ChatCompletionMessage, ChatCompletionRequest, ChatCompletionRoles};

pub async fn prompt_model(prompt: &str) -> Result<String, ModelError> {
    let api_key = env::var("GROQ_API_KEY").map_err(|_| ModelError::UnableToConnectToModel)?;


    let client = AsyncGroqClient::new(
        api_key,
        Some("https://api.groq.com/openai/v1".to_string()),
    ).await;

    let messages = vec![ChatCompletionMessage {
        role: ChatCompletionRoles::User,
        content: prompt.to_string(),
        name: None,
    }];

    let response = client
        .chat_completion(ChatCompletionRequest::new("llama-3.1-8b-instant", messages))
        .await
        .map_err(|_| ModelError::UnableToConnectToModel)?;

    Ok(response.choices[0].message.content.clone())
}

#[derive(Debug)]
pub enum ModelError{
    UnableToParseModelResponse,
    UnableToConnectToModel,
}