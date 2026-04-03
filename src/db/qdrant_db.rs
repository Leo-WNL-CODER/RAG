use qdrant_client::{Qdrant, qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder}};

use crate::db::db_error::{DbErrors};

// The Rust client uses Qdrant's gRPC interface

pub async fn db_create_collection(client:&Qdrant)->Result<String,DbErrors>{
    let collection_name="test_collection";
    let client_exists = match client.collection_exists(collection_name).await {
        Ok(exists) => exists,
        Err(e) => {
            return Err(DbErrors::DbFailedToRespond(
                format!("Error in db: {:?}", e)
            ));
        }
    };
    
    if !client_exists {
        let Ok(_)=client
        .create_collection(
            CreateCollectionBuilder::new("test_collection")
                .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine))
        )
        .await else{
            return Err(DbErrors::FailedToInitializeDatabase("Error while creating vector database".to_string()));
        };

    };
    Ok("QUADRANT_UP".to_string())
}
