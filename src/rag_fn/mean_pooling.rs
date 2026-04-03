use ndarray::Array2;

pub fn mean_pooling(token_emb: &ndarray::Array2<f32>, attention_mask: &[i64]) -> Vec<f32> {
    let hidden = token_emb.shape()[1];
    let mut sum = vec![0.0; hidden];
    let mut count = 0.0;

    for (i, mask) in attention_mask.iter().enumerate() {
        if *mask == 1 {
            count += 1.0;
            let row = token_emb.row(i);
            for h in 0..hidden {
                sum[h] += row[h];
            }
        }
    }

    // Avoid divide-by-zero
    if count > 0.0 {
        for v in &mut sum {
            *v /= count;
        }
    }

    // L2 normalize
    let norm = sum.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut sum {
            *v /= norm;
        }
    }

    sum
}
