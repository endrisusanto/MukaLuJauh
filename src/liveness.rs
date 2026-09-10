use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LivenessMode {
    Light,
    Heavy,
    Off,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LivenessDecision {
    Confirmed { cue: String },
    Denied { reason: String },
    Pending,
}

#[derive(Debug, Clone)]
pub struct LivenessFrameInput {
    pub frame_timestamp_ms: u64,
    pub eye_aspect_ratio: f32,
    pub head_yaw: f32,
    pub head_pitch: f32,
    pub head_roll: f32,
    pub specular_glare_ratio: f32,
    pub bezel_edge_score: f32,
}

pub struct LivenessAnalyzer {
    pub mode: LivenessMode,
    history: Vec<LivenessFrameInput>,
    max_history: usize,
    confirmed: bool,
}

impl LivenessAnalyzer {
    pub fn new(mode: LivenessMode) -> Self {
        Self {
            mode,
            history: Vec::with_capacity(30),
            max_history: 25,
            confirmed: mode == LivenessMode::Off,
        }
    }

    pub fn reset(&mut self) {
        self.history.clear();
        self.confirmed = self.mode == LivenessMode::Off;
    }

    /// Evaluates anti-spoofing criteria based on temporal frames
    pub fn observe(&mut self, frame: LivenessFrameInput) -> LivenessDecision {
        if self.mode == LivenessMode::Off || self.confirmed {
            return LivenessDecision::Confirmed {
                cue: "Liveness disabled/already confirmed".to_string(),
            };
        }

        // 1. Check for immediate hard denial: Device bezel reflection / smartphone screen edge
        let bezel_threshold = match self.mode {
            LivenessMode::Heavy => 0.45,
            _ => 0.65,
        };
        if frame.bezel_edge_score > bezel_threshold {
            return LivenessDecision::Denied {
                reason: "Suspicious rectangular screen bezel detected (Possible phone/screen spoof)"
                    .to_string(),
            };
        }

        // 2. Check for abnormal static glare cue
        if frame.specular_glare_ratio > 0.85 {
            return LivenessDecision::Denied {
                reason: "Unnatural uniform glass glare detected".to_string(),
            };
        }

        self.history.push(frame);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Need at least 5 frames to observe micro-motion / non-rigid 3D geometry
        if self.history.len() < 5 {
            return LivenessDecision::Pending;
        }

        // 3. Confirm micro-motion / blink / natural 3D depth variance
        let mut min_ear = f32::MAX;
        let mut max_ear = f32::MIN;
        let mut yaw_variance = 0.0f32;
        let mut pitch_variance = 0.0f32;

        let avg_yaw: f32 = self.history.iter().map(|f| f.head_yaw).sum::<f32>() / self.history.len() as f32;
        let avg_pitch: f32 = self.history.iter().map(|f| f.head_pitch).sum::<f32>() / self.history.len() as f32;

        for f in &self.history {
            if f.eye_aspect_ratio < min_ear { min_ear = f.eye_aspect_ratio; }
            if f.eye_aspect_ratio > max_ear { max_ear = f.eye_aspect_ratio; }
            yaw_variance += (f.head_yaw - avg_yaw).powi(2);
            pitch_variance += (f.head_pitch - avg_pitch).powi(2);
        }

        let ear_delta = max_ear - min_ear;
        let angular_motion = (yaw_variance + pitch_variance).sqrt();

        // Blink detection (EAR delta) or micro head motion
        if ear_delta > 0.08 {
            self.confirmed = true;
            return LivenessDecision::Confirmed {
                cue: "Natural eye blink detected".to_string(),
            };
        }

        let motion_threshold = match self.mode {
            LivenessMode::Heavy => 0.04,
            LivenessMode::Light => 0.015,
            LivenessMode::Off => 0.0,
        };

        if angular_motion > motion_threshold {
            self.confirmed = true;
            return LivenessDecision::Confirmed {
                cue: "3D head micro-motion confirmed".to_string(),
            };
        }

        LivenessDecision::Pending
    }
}
