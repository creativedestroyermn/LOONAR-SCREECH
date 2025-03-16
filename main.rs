//! LOONAR SCREECH Core System - MIT License
//! Copyright © 2025 Clarke Macbeth - University of Minnesota

#[cfg(not(target_arch = "wasm32"))]
use cpal::traits::{HostTrait, StreamTrait};
use rodio::{source::Source, Decoder, OutputStream};
use serde::Deserialize;
use std::{
    collections::HashSet,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use tokio::{
    sync::mpsc,
    time::{interval, Interval},
};

#[derive(Debug, Deserialize, Clone)]
pub struct SystemConfig {
    #[serde(default = "default_threshold")]
    pub db_threshold: f32,
    #[serde(default = "default_danger_sounds")]
    pub danger_patterns: HashSet<String>,
    #[serde(default = "default_alert_radius")]
    pub alert_radius_miles: f32,
    #[serde(default = "default_false_positive_rate")]
    pub fp_rate: f32,
}

// ... [FULL IMPLEMENTATION FROM PREVIOUS MESSAGES] ...

fn default_danger_sounds() -> HashSet<String> {
    ["gunshot", "glass_break", "scream"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}
