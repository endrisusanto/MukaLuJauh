use crate::model::{EmbeddingVector, FaceIdentity, ScoredIdentity};

pub struct FaceRecognitionPipeline {
    pub default_threshold: f32,
}

impl Default for FaceRecognitionPipeline {
    fn default() -> Self {
        Self {
            default_threshold: 0.60,
        }
    }
}

impl FaceRecognitionPipeline {
    pub fn new(threshold: f32) -> Self {
        Self {
            default_threshold: threshold,
        }
    }

    /// Computes cosine similarity between two normalized 512-d feature vectors
    /// Returns value in range [-1.0, 1.0] where 1.0 means exact identical match.
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;

        for (x, y) in a.iter().zip(b.iter()) {
            dot_product += x * y;
            norm_a += x * x;
            norm_b += y * y;
        }

        let denominator = norm_a.sqrt() * norm_b.sqrt();
        if denominator < 1e-6 {
            return 0.0;
        }

        (dot_product / denominator).clamp(-1.0, 1.0)
    }

    /// Scores a candidate face embedding against all active identities
    pub fn score_embedding(
        &self,
        candidate: &EmbeddingVector,
        identities: &[FaceIdentity],
    ) -> Vec<ScoredIdentity> {
        let mut scores = Vec::new();

        for identity in identities {
            if !identity.is_active {
                continue;
            }

            // Score against the precomputed centroid if available
            if let Some(centroid) = identity.centroid() {
                let similarity = Self::cosine_similarity(candidate, &centroid);
                scores.push(ScoredIdentity {
                    identity_name: identity.name.clone(),
                    identity_id: identity.id.clone(),
                    similarity,
                });
            } else if !identity.samples.is_empty() {
                // Fallback: max similarity against individual samples
                let mut max_sim = -1.0f32;
                for sample in &identity.samples {
                    let sim = Self::cosine_similarity(candidate, &sample.embedding);
                    if sim > max_sim {
                        max_sim = sim;
                    }
                }
                scores.push(ScoredIdentity {
                    identity_name: identity.name.clone(),
                    identity_id: identity.id.clone(),
                    similarity: max_sim,
                });
            }
        }

        // Sort descending by similarity
        scores.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        scores
    }

    /// Finds the best match exceeding the confidence threshold
    pub fn find_best_match(
        &self,
        candidate: &EmbeddingVector,
        identities: &[FaceIdentity],
        threshold: Option<f32>,
    ) -> Option<ScoredIdentity> {
        let th = threshold.unwrap_or(self.default_threshold);
        let scored = self.score_embedding(candidate, identities);

        if let Some(best) = scored.first() {
            if best.similarity >= th {
                return Some(best.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_cosine_similarity() {
        let v1 = vec![0.5f32, 0.5, 0.5, 0.5];
        let v2 = vec![0.5f32, 0.5, 0.5, 0.5];
        let sim = FaceRecognitionPipeline::cosine_similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_orthogonal_cosine_similarity() {
        let v1 = vec![1.0f32, 0.0];
        let v2 = vec![0.0f32, 1.0];
        let sim = FaceRecognitionPipeline::cosine_similarity(&v1, &v2);
        assert!(sim.abs() < 1e-4);
    }
}
