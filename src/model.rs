use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// 512-dimensional feature embedding vector extracted by ArcFace model
pub type EmbeddingVector = Vec<f32>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrolledSample {
    pub id: String,
    pub pose_label: String, // e.g. "Center", "Look Left", "Look Right", "Tilt Up", "Tilt Down"
    pub embedding: EmbeddingVector,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceIdentity {
    pub id: String,
    pub name: String,
    pub samples: Vec<EnrolledSample>,
    pub is_active: bool,
    pub created_at: u64,
}

impl FaceIdentity {
    pub fn new(name: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: format!("face_{}_{}", name.to_lowercase().replace(' ', "_"), now),
            name: name.to_string(),
            samples: Vec::new(),
            is_active: true,
            created_at: now,
        }
    }

    /// Calculate the normalized centroid (mean vector) across all active samples
    pub fn centroid(&self) -> Option<EmbeddingVector> {
        if self.samples.is_empty() {
            return None;
        }

        let dim = self.samples[0].embedding.len();
        let mut sum = vec![0.0f32; dim];

        for sample in &self.samples {
            for (i, val) in sample.embedding.iter().enumerate() {
                sum[i] += val;
            }
        }

        let count = self.samples.len() as f32;
        let mut mean: Vec<f32> = sum.into_iter().map(|v| v / count).collect();

        // L2 Normalize the centroid vector
        let norm: f32 = mean.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-6 {
            for x in &mut mean {
                *x /= norm;
            }
        }

        Some(mean)
    }
}

#[derive(Debug, Clone)]
pub struct ScoredIdentity {
    pub identity_name: String,
    pub identity_id: String,
    pub similarity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScanOutcome {
    Matched { identity_name: String, similarity: f32 },
    ConsistentlyWrongFace,
    SpoofSuspected { reason: String },
    NoFaceDetected,
    Timeout,
}
