//! Cross-platform main.rs - Works on all supported OS

#[cfg(unix)]
use libc;
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
pub struct Config {
    #[serde(default = "default_threshold")]
    pub db_threshold: f32,
    #[serde(default = "default_danger_sounds")]
    pub danger_patterns: HashSet<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_threshold: 85.0,
            danger_patterns: ["gunshot", "glass_break", "scream"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

pub struct AudioGuardian {
    config: Config,
    alert_active: AtomicBool,
}

impl AudioGuardian {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            alert_active: AtomicBool::new(false),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let (tx, mut rx) = mpsc::channel(32);
        
        #[cfg(target_os = "android")]
        let host = cpal::host_from_id(cpal::HostId::Jack).unwrap();
        #[cfg(not(target_os = "android"))]
        let host = cpal::default_host();

        let input_device = host.default_input_device().unwrap();
        let config = input_device.default_input_config().unwrap();

        let input_stream = input_device.build_input_stream(
            &config.config(),
            move |data, _: &_| tx.blocking_send(data.to_vec()).unwrap(),
            |err| eprintln!("Audio error: {}", err),
            None,
        )?;

        input_stream.play()?;
        let mut last_alert = Instant::now();

        loop {
            if let Some(samples) = rx.recv().await {
                let avg = samples.iter().map(|&s| s.abs()).sum::<f32>() / samples.len() as f32;
                if 20.0 * avg.log10() > self.config.db_threshold {
                    self.trigger_alerts().await;
                }
            }
        }
    }

    async fn trigger_alerts(&self) {
        // Cross-platform alerts
        #[cfg(target_os = "android")]
        android_alert().await;
        
        #[cfg(windows)]
        windows_alert().await;
        
        #[cfg(unix)]
        unix_alert().await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    AudioGuardian::new(Config::default()).run().await
}