use ndarray::{ArrayViewD, Axis, Ix3};
use ort::{session::Session, value::Tensor};
use tokenizers::Encoding;

use crate::rag_fn::mean_pooling::mean_pooling;

const CHUNK_SIZE:usize=200;
const OVERLAP:usize=50;

pub fn chunk_text(enc: Encoding, session: &mut Session) -> Result<Vec<(Vec<f32>, (usize, usize))>,ChunkError> {
    let mut result = Vec::new();

    let input_ids: Vec<i64> = enc.get_ids().iter().map(|x| *x as i64).collect();
    let attention_mask: Vec<i64> = enc.get_attention_mask().iter().map(|x| *x as i64).collect();
    let offsets = enc.get_offsets().to_vec();
    let token_type_ids = vec![0i64; input_ids.len()];

    // Chunk settings
    
    let cls_id = input_ids[0];
    let sep_id = input_ids[input_ids.len() - 1];

    let mut start = 1;

    while start < input_ids.len() - 1 {
        let end = (start + CHUNK_SIZE).min(input_ids.len() - 1);

        let (start_char, end_char) = (offsets[start].0, offsets[end - 1].1);

        // Extract sub-slice
        let mut ids = input_ids[start..end].to_vec();
        let mut mask = attention_mask[start..end].to_vec();
        let mut types = token_type_ids[start..end].to_vec();

        // Add CLS + SEP
        ids.insert(0, cls_id);
        ids.push(sep_id);

        mask.insert(0, 1);
        mask.push(1);

        types.insert(0, 0);
        types.push(0);

        let shape = [1, ids.len()];

        let Ok(ids_val) = Tensor::from_array((shape, ids)) else {
            return Err(ChunkError::FailedToTokenize)
        };
        let Ok(mask_val) = Tensor::from_array((shape, mask.clone())) else {
            return Err(ChunkError::FailedToTokenize)
        };
        let Ok(types_val) = Tensor::from_array((shape, types)) else {
            return Err(ChunkError::FailedToTokenize)
        };

        let Ok(outputs) = session
            .run(ort::inputs![
                "input_ids" => ids_val,
                "attention_mask" => mask_val,
                "token_type_ids" => types_val
            ]) else {
                return Err(ChunkError::FailedToEmbed)
            };

        let Ok(emb_3d)= outputs[0].try_extract_array() else{
            return Err(ChunkError::FailedToEmbed)
        };
        let Ok(emb_3d)= emb_3d.into_dimensionality::<Ix3>() else{
            return Err(ChunkError::FailedToEmbed)
        };
        let emb_2d = emb_3d.index_axis(Axis(0), 0).to_owned();

        let pooled = mean_pooling(&emb_2d, &mask);

        result.push((pooled, (start_char, end_char)));

        // Move window
        if end == input_ids.len() - 1 {
            break;
        }

        start += CHUNK_SIZE - OVERLAP;
    }

    Ok(result)
}

#[derive(Debug)]
pub enum ChunkError{
    FailedToTokenize,
    FailedToEmbed,
    FailedToChunk,
}