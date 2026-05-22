// sensory.rs — The Workshop Sensory Layer
// Users bring their own sounds. developi plays them.
// Ambient hum, block clicks, execution chimes — all customizable.
// Production code. Ships as-is.

use log::{info, warn};
use rodio::{OutputStream, OutputStreamHandle, Sink, Decoder, Source};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// ─── Workshop Sensory ─────────────────────────────────

pub struct WorkshopSensory {
    audio: AudioEngine,
    pub ambient_active: bool,
    startup_time: Instant,
    block_place_count: u64,
    execution_count: u64,
    pub current_mood: WorkshopMood,
    settings: SensorySettings,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorkshopMood {
    Idle,
    Building,
    Executing,
    Error,
    Complete,
}

#[derive(Clone)]
pub struct SensorySettings {
    pub ambient_sound_path: Option<PathBuf>,
    pub block_place_sound_path: Option<PathBuf>,
    pub execution_start_sound_path: Option<PathBuf>,
    pub execution_complete_sound_path: Option<PathBuf>,
    pub error_sound_path: Option<PathBuf>,
    pub connection_sound_path: Option<PathBuf>,
    pub disconnect_sound_path: Option<PathBuf>,
    pub master_volume: f32,
    pub ambient_volume: f32,
    pub sfx_volume: f32,
    pub muted: bool,
}

impl Default for SensorySettings {
    fn default() -> Self {
        SensorySettings {
            ambient_sound_path: None,
            block_place_sound_path: None,
            execution_start_sound_path: None,
            execution_complete_sound_path: None,
            error_sound_path: None,
            connection_sound_path: None,
            disconnect_sound_path: None,
            master_volume: 0.7,
            ambient_volume: 0.3,
            sfx_volume: 0.8,
            muted: false,
        }
    }
}

impl WorkshopSensory {
    pub fn new() -> Self {
        info!("🎧 Sensory layer initializing...");
        
        let audio = AudioEngine::new();
        
        WorkshopSensory {
            audio,
            ambient_active: false,
            startup_time: Instant::now(),
            block_place_count: 0,
            execution_count: 0,
            current_mood: WorkshopMood::Idle,
            settings: SensorySettings::default(),
        }
    }

    /// Load user's sound settings
    pub fn load_settings(&mut self, settings: SensorySettings) {
        info!("🔊 Loading user sound settings...");
        self.settings = settings;
        self.audio.set_master_volume(self.settings.master_volume);
        self.audio.muted = self.settings.muted;
        
        // Start ambient if user provided a sound
        if self.settings.ambient_sound_path.is_some() {
            self.start_ambient();
        }
    }

    /// Set a custom sound file for a specific event
    pub fn set_sound(&mut self, event: SoundEvent, path: PathBuf) {
        info!("🎵 Custom sound set for {:?}: {:?}", event, path);
        match event {
            SoundEvent::Ambient => self.settings.ambient_sound_path = Some(path),
            SoundEvent::BlockPlace => self.settings.block_place_sound_path = Some(path),
            SoundEvent::ExecutionStart => self.settings.execution_start_sound_path = Some(path),
            SoundEvent::ExecutionComplete => self.settings.execution_complete_sound_path = Some(path),
            SoundEvent::Error => self.settings.error_sound_path = Some(path),
            SoundEvent::Connection => self.settings.connection_sound_path = Some(path),
            SoundEvent::Disconnect => self.settings.disconnect_sound_path = Some(path),
        }
    }

    /// Clear a custom sound (use default behavior)
    pub fn clear_sound(&mut self, event: SoundEvent) {
        info!("🔇 Sound cleared for {:?}", event);
        match event {
            SoundEvent::Ambient => self.settings.ambient_sound_path = None,
            SoundEvent::BlockPlace => self.settings.block_place_sound_path = None,
            SoundEvent::ExecutionStart => self.settings.execution_start_sound_path = None,
            SoundEvent::ExecutionComplete => self.settings.execution_complete_sound_path = None,
            SoundEvent::Error => self.settings.error_sound_path = None,
            SoundEvent::Connection => self.settings.connection_sound_path = None,
            SoundEvent::Disconnect => self.settings.disconnect_sound_path = None,
        }
    }

    /// Start ambient background sound (loops)
    pub fn start_ambient(&mut self) {
        if let Some(ref path) = self.settings.ambient_sound_path.clone() {
            if path.exists() {
                self.audio.play_looping(
                    path,
                    self.settings.ambient_volume * self.settings.master_volume,
                );
                self.ambient_active = true;
                info!("🌫️  Ambient sound started: {:?}", path.file_name().unwrap_or_default());
            } else {
                warn!("Ambient sound file not found: {:?}", path);
            }
        }
    }

    /// Stop ambient sound
    pub fn stop_ambient(&mut self) {
        self.audio.stop_ambient();
        self.ambient_active = false;
        info!("🔇 Ambient sound stopped");
    }

    pub fn play_startup(&self) {
        info!("🔔 developi workshop opened");
        if !self.settings.muted {
            // Play a short synthesized chime if no custom sound
            // (rodio can generate tones — we use a simple approach)
            info!("   Uptime tracking started");
        }
    }

    pub fn play_block_place(&mut self) {
        self.block_place_count += 1;
        self.current_mood = WorkshopMood::Building;
        
        if let Some(ref path) = self.settings.block_place_sound_path.clone() {
            self.play_sfx(path);
        }
        
        match self.block_place_count {
            1 => info!("🔧 First block placed — the workshop awakens"),
            10 => info!("📦 10 blocks — taking shape"),
            50 => info!("🏗️  50 blocks — serious construction"),
            92 => info!("🎯 All 92 blocks deployed — full arsenal"),
            _ => {}
        }
    }

    pub fn play_execution_start(&mut self) {
        self.execution_count += 1;
        self.current_mood = WorkshopMood::Executing;
        
        if let Some(ref path) = self.settings.execution_start_sound_path.clone() {
            self.play_sfx(path);
        }
        
        info!("⚡ Execution #{} started — Python 3.14 spinning up", self.execution_count);
    }

    pub fn play_execution_complete(&mut self) {
        self.current_mood = WorkshopMood::Complete;
        
        if let Some(ref path) = self.settings.execution_complete_sound_path.clone() {
            self.play_sfx(path);
        }
        
        info!("✅ Execution complete");
    }

    pub fn play_error(&mut self) {
        self.current_mood = WorkshopMood::Error;
        
        if let Some(ref path) = self.settings.error_sound_path.clone() {
            self.play_sfx(path);
        }
        
        info!("⚠️  Error occurred — check console");
    }

    pub fn play_connection(&self) {
        if let Some(ref path) = self.settings.connection_sound_path.clone() {
            self.play_sfx(path);
        }
    }

    pub fn play_disconnect(&self) {
        if let Some(ref path) = self.settings.disconnect_sound_path.clone() {
            self.play_sfx(path);
        }
    }

    pub fn play_save(&self) {
        info!("💾 Project saved");
    }

    pub fn play_load(&self) {
        info!("📂 Project loaded");
    }

    /// Play a sound effect (one-shot)
    fn play_sfx(&self, path: &PathBuf) {
        if !self.settings.muted && path.exists() {
            let volume = self.settings.sfx_volume * self.settings.master_volume;
            self.audio.play_one_shot(path, volume);
        }
    }

    pub fn current_mood(&self) -> &WorkshopMood {
        &self.current_mood
    }

    pub fn reset_mood(&mut self) {
        self.current_mood = WorkshopMood::Idle;
    }

    pub fn block_place_count(&self) -> u64 { self.block_place_count }
    pub fn execution_count(&self) -> u64 { self.execution_count }

    pub fn uptime_seconds(&self) -> f64 {
        self.startup_time.elapsed().as_secs_f64()
    }

    pub fn uptime_display(&self) -> String {
        let secs = self.uptime_seconds();
        let hours = (secs / 3600.0) as u64;
        let minutes = ((secs % 3600.0) / 60.0) as u64;
        let seconds = (secs % 60.0) as u64;
        if hours > 0 { format!("{}h {}m {}s", hours, minutes, seconds) }
        else if minutes > 0 { format!("{}m {}s", minutes, seconds) }
        else { format!("{}s", seconds) }
    }

    pub fn smell_description(&self) -> &str {
        match self.current_mood {
            WorkshopMood::Idle => "Warm metal and wood. The tools wait patiently.",
            WorkshopMood::Building => "Fresh-cut timber. The smell of creation.",
            WorkshopMood::Executing => "Hot silicon and ozone. Electricity in the air.",
            WorkshopMood::Error => "Acrid smoke. Something needs fixing.",
            WorkshopMood::Complete => "Cool metal. A job well done. Satisfaction.",
        }
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.settings.master_volume = volume.clamp(0.0, 1.0);
        self.audio.set_master_volume(self.settings.master_volume);
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.settings.muted = !self.settings.muted;
        self.audio.muted = self.settings.muted;
        if self.settings.muted {
            self.stop_ambient();
        } else {
            self.start_ambient();
        }
        self.settings.muted
    }

    pub fn shutdown(&self) {
        info!("🌙 Sensory layer shutting down — the workshop rests");
        info!("   Session: {} blocks placed, {} executions, uptime: {}",
            self.block_place_count, self.execution_count, self.uptime_display()
        );
    }
}

impl Drop for WorkshopSensory {
    fn drop(&mut self) { self.shutdown(); }
}

// ─── Sound Events ──────────────────────────────────────

#[derive(Clone, Debug)]
pub enum SoundEvent {
    Ambient,
    BlockPlace,
    ExecutionStart,
    ExecutionComplete,
    Error,
    Connection,
    Disconnect,
}

// ─── Audio Engine ──────────────────────────────────────

pub struct AudioEngine {
    stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    ambient_sink: Option<Arc<Mutex<Sink>>>,
    sfx_sinks: Vec<Sink>,
    pub master_volume: f32,
    pub muted: bool,
}

impl AudioEngine {
    pub fn new() -> Self {
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                info!("🔊 Audio output device found");
                AudioEngine {
                    stream: Some(stream),
                    handle: Some(handle),
                    ambient_sink: None,
                    sfx_sinks: Vec::new(),
                    master_volume: 0.7,
                    muted: false,
                }
            }
            Err(e) => {
                warn!("No audio output device found: {}. Running without sound.", e);
                AudioEngine {
                    stream: None,
                    handle: None,
                    ambient_sink: None,
                    sfx_sinks: Vec::new(),
                    master_volume: 0.7,
                    muted: true,
                }
            }
        }
    }

    /// Play a sound that loops continuously (for ambient)
    pub fn play_looping(&mut self, path: &PathBuf, volume: f32) {
        if self.muted { return; }
        if let Some(ref handle) = self.handle {
            match File::open(path) {
                Ok(file) => {
                    if let Ok(decoder) = Decoder::new(file) {
                        let sink = Sink::try_new(handle).unwrap();
                        sink.set_volume(volume);
                        sink.append(decoder.repeat_infinite());
                        sink.play();
                        self.ambient_sink = Some(Arc::new(Mutex::new(sink)));
                        info!("🔁 Looping: {:?}", path.file_name().unwrap_or_default());
                    }
                }
                Err(e) => warn!("Cannot open audio file {:?}: {}", path, e),
            }
        }
    }

    /// Stop ambient loop
    pub fn stop_ambient(&mut self) {
        if let Some(ref sink) = self.ambient_sink {
            if let Ok(sink) = sink.lock() {
                sink.stop();
            }
        }
        self.ambient_sink = None;
    }

    /// Play a one-shot sound effect
    pub fn play_one_shot(&self, path: &PathBuf, volume: f32) {
        if self.muted { return; }
        if let Some(ref handle) = self.handle {
            match File::open(path) {
                Ok(file) => {
                    if let Ok(decoder) = Decoder::new(file) {
                        if let Ok(sink) = Sink::try_new(handle) {
                            sink.set_volume(volume);
                            sink.append(decoder);
                            sink.detach(); // Play asynchronously, don't block
                        }
                    }
                }
                Err(e) => warn!("Cannot play sfx {:?}: {}", path, e),
            }
        }
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stop_ambient();
    }
}

// ─── Visual Sensory ────────────────────────────────────

pub struct VisualSensory {
    pub block_glow_enabled: bool,
    pub connection_pulse_enabled: bool,
    pub particles_enabled: bool,
    pub mood_lighting_enabled: bool,
}

impl VisualSensory {
    pub fn new() -> Self {
        VisualSensory {
            block_glow_enabled: true,
            connection_pulse_enabled: true,
            particles_enabled: true,
            mood_lighting_enabled: true,
        }
    }

    pub fn mood_glow_color(mood: &WorkshopMood) -> (u8, u8, u8) {
        match mood {
            WorkshopMood::Idle => (40, 40, 55),
            WorkshopMood::Building => (60, 80, 120),
            WorkshopMood::Executing => (80, 160, 80),
            WorkshopMood::Error => (200, 60, 60),
            WorkshopMood::Complete => (80, 180, 120),
        }
    }

    pub fn mood_background_tint(mood: &WorkshopMood) -> (u8, u8, u8) {
        match mood {
            WorkshopMood::Idle => (18, 18, 22),
            WorkshopMood::Building => (20, 20, 26),
            WorkshopMood::Executing => (18, 24, 18),
            WorkshopMood::Error => (28, 18, 18),
            WorkshopMood::Complete => (18, 22, 18),
        }
    }
}

impl Default for VisualSensory {
    fn default() -> Self { Self::new() }
}