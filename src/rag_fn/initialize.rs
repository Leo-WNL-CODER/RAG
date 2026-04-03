// use anyhow::Result;
use ort::session::Session;
use tokenizers::Tokenizer;



pub fn initialize() -> Result<(Session,Tokenizer),InitError>{
    let Ok(_)=ort::init().commit() else{
        return Err(InitError::FailedToLoadModel)
    };

    let Ok(session_builder) = Session::builder() else{
        return Err(InitError::FailedToLoadModel)
    };

    let Ok(session)=session_builder.commit_from_file("models/model.onnx") else{
            return Err(InitError::FailedToLoadModel)
    };

    let Ok(tokenizer) = Tokenizer::from_file("models/tokenizer.json") else{
        return Err(InitError::FailedToLoadTokenizer)
    };

    Ok((session, tokenizer ))
}

#[derive(Debug)]
pub enum InitError{
    FailedToLoadTokenizer,
    FailedToLoadModel,
}
