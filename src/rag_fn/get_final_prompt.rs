use qdrant_client::qdrant::{ScoredPoint, value::Kind};

pub fn get_prompt(
    history: &str,
    search_response: &Vec<ScoredPoint>,
    user_query: &String,
) -> String {
    let conversation_summary = if !history.trim().is_empty() {
        format!(
            r#"
----------------------------------------
Conversation Summary:
{}
"#,
            history
        )
    } else {
        String::new()
    };

    let retrieved_context = {
        let mut chunks = String::new();
        for (idx, point) in search_response.iter().enumerate() {
            if let Some(Kind::StringValue(text)) = &point.payload["text"].kind {
                chunks.push_str(&format!("{}. {}\n", idx + 1, text));
            }
        }

        format!(
            r#"
----------------------------------------
Retrieved Context:
{}
"#,
            chunks
        )
    };

    format!(
        r#"System Instruction:
You are a factual, retrieval-grounded assistant.

Your task is to answer the user’s question using ONLY:
1. The Retrieved Context (highest priority)
2. The Conversation Summary (secondary context)

If the answer is not explicitly supported by these sources, respond with:
"I do not know based on the given context."
{}
{}
----------------------------------------
User Question:
{}

----------------------------------------
Answering Rules:
- Use Retrieved Context first; use Conversation Summary only if needed
- Do NOT use outside knowledge
- Do NOT hallucinate or assume missing facts
- Be concise, clear, and factual
- If the answer cannot be determined, say you do not know

Provide the answer below:
"#,
        conversation_summary,
        retrieved_context,
        user_query
    )
}
