//! LOONAR SCREECH Core System - MIT License
//! Copyright © 2025 Clarke Macbeth

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

// Configuration module
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

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            db_threshold: default_threshold(),
            danger_patterns: default_danger_sounds(),
            alert_radius_miles: default_alert_radius(),
            fp_rate: default_false_positive_rate(),
        }
    }
}

// Audio processing core
pub struct AudioGuardian {
    config: SystemConfig,
    alert_active: AtomicBool,
    sample_buffer: Vec<f32>,
}

impl AudioGuardian {
    pub fn new(config: SystemConfig) -> Self {
        Self {
            config,
            alert_active: AtomicBool::new(false),
            sample_buffer: Vec::with_capacity(4096),
        }
    }

    pub async fn monitor(&mut self) -> anyhow::Result<()> {
        let (tx, mut rx) = mpsc::channel(32);
        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device"))?;

        let config = input_device
            .default_input_config()
            .map_err(|e| anyhow::anyhow!(e))?;

        let input_stream = input_device.build_input_stream(
            &config.config(),
            move |data: &[f32], _: &_| {
                let _ = tx.blocking_send(data.to_vec());
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        input_stream.play()?;
        let mut last_alert = Instant::now();

        loop {
            if let Some(samples) = rx.recv().await {
                self.process_samples(&samples).await;

                if self.alert_active.load(Ordering::SeqCst) {
                    if last_alert.elapsed() > Duration::from_secs(5) {
                        self.trigger_alerts().await;
                        last_alert = Instant::now();
                    }
                    self.alert_active.store(false, Ordering::SeqCst);
                }
            }
        }
    }

    async fn process_samples(&mut self, samples: &[f32]) {
        // Simplified detection logic (MLP classifier planned)
        let avg_volume = samples.iter().map(|&s| s.abs()).sum::<f32>() / samples.len() as f32;
        let simulated_db = 20.0 * avg_volume.log10();

        if simulated_db > self.config.db_threshold && rand::random::<f32>() > self.config.fp_rate {
            self.alert_active.store(true, Ordering::SeqCst);
        }
    }

    async fn trigger_alerts(&self) {
        // Multi-channel alert system
        tokio::join!(
            self.play_alert_sound(),
            self.flash_visual_alert(),
            self.send_sms_alerts()
        );
    }

    async fn play_alert_sound(&self) {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sound_data = include_bytes!("../assets/loon_alert.wav");
        let source = Decoder::new(std::io::Cursor::new(sound_data)).unwrap();
        handle.play_raw(source.convert_samples()).ok();
    }

    async fn flash_visual_alert(&self) {
        println!("🚨 LOONAR ALERT ACTIVATED - SEEK SHELTER 🚨");
    }

    async fn send_sms_alerts(&self) {
        // Stub for SMS integration
        println!("SMS alerts sent to registered devices within {} mile radius", 
               self.config.alert_radius_miles);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config().await;
    let mut guardian = AudioGuardian::new(config);
    
    let mut health_check = interval(Duration::from_secs(60));
    tokio::select! {
        _ = guardian.monitor() => {},
        _ = async {
            loop {
                health_check.tick().await;
                println!("System health: OK ({}MB RAM)", resident_memory());
            }
        } => {},
    }

    Ok(())
}

// Helper functions
async fn load_config() -> SystemConfig {
    // Future TOML implementation
    SystemConfig::default()
}

fn resident_memory() -> usize {
    let pages = unsafe {
        libc::sysconf(libc::_SC_PHYS_PAGES) * libc::sysconf(libc::_SC_PAGE_SIZE)
    };
    (pages / 1024 / 1024) as usize
}

fn default_threshold() -> f32 { 85.0 }
fn default_alert_radius() -> f32 { 5.0 }
fn default_false_positive_rate() -> f32 { 0.05 }
fn default_danger_sounds() -> HashSet<String> {
    ["gunshot", "glass_break", "scream"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}