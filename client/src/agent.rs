use serde::{Deserialize, Serialize};
use std::fs;
use ndarray::Array1;
use ndarray_npy::read_npy;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    pub layer_dims: Vec<usize>,
    pub state_dim: usize,
    pub n_actions: usize,
    pub team: String,
}

pub struct NeuralAgent {
    pub config: ModelConfig,
    pub genome: Vec<f32>,
}

pub fn load_config(path: &str) -> anyhow::Result<ModelConfig> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn load_genome(path: &str) -> anyhow::Result<Vec<f32>> {
    match read_npy::<_, Array1<f32>>(path) {
        Ok(arr) => Ok(arr.to_vec()),
        Err(e32) => match read_npy::<_, Array1<f64>>(path) {
            Ok(arr64) => Ok(arr64.iter().map(|&v| v as f32).collect()),
            Err(e64) => Err(anyhow::anyhow!(
                "Failed to read numpy file '{}' as f32 ({}) or f64 ({})",
                path,
                e32,
                e64
            )),
        },
    }
}

fn linear(weights: &[f32], bias: &[f32], input: &[f32], out_dim: usize, in_dim: usize) -> Vec<f32> {
    let mut output = vec![0.0f32; out_dim];
    for o in 0..out_dim {
        let mut sum = bias[o];
        for i in 0..in_dim {
            sum += weights[o * in_dim + i] * input[i];
        }
        output[o] = sum;
    }
    output
}

fn relu(v: &mut [f32]) {
    for x in v.iter_mut() {
        *x = x.max(0.0);
    }
}

pub fn masked_argmax(logits: &[f32], valid_mask: &[bool]) -> usize {
    let mut best_idx = 0;
    let mut best_score = f32::NEG_INFINITY;
    for (i, (&l, &v)) in logits.iter().zip(valid_mask.iter()).enumerate() {
        let score = if v { l } else { f32::NEG_INFINITY };
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }
    best_idx
}

fn expected_genome_size(dims: &[usize]) -> usize {
    dims.windows(2).map(|w| w[0] * w[1] + w[1]).sum()
}

impl NeuralAgent {
    pub fn load(json_path: &str, npy_path: &str) -> anyhow::Result<Self> {
        let config = load_config(json_path)?;
        let genome = load_genome(npy_path)?;
        let expected = expected_genome_size(&config.layer_dims);
        if genome.len() != expected {
            anyhow::bail!("Expected genome size {}, got {}", expected, genome.len());
        }
        Ok(Self { config, genome })
    }

    pub fn forward(&self, state: &[f32]) -> Vec<f32> {
        let mut x = state.to_vec();
        let mut idx = 0;
        for layer in 0..self.config.layer_dims.len() - 1 {
            let in_dim = self.config.layer_dims[layer];
            let out_dim = self.config.layer_dims[layer + 1];
            let weight_count = in_dim * out_dim;
            let weights = &self.genome[idx..idx + weight_count];
            idx += weight_count;
            let bias = &self.genome[idx..idx + out_dim];
            idx += out_dim;
            let mut y = linear(weights, bias, &x, out_dim, in_dim);
            let is_last = layer == self.config.layer_dims.len() - 2;
            if !is_last {
                relu(&mut y);
            }
            x = y;
        }
        x
    }

    pub fn act(&self, state: &[f32], valid_mask: &[bool]) -> usize {
        let logits = self.forward(state);
        masked_argmax(&logits, valid_mask)
    }
}
