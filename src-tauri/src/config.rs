use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub whisper_model_path: String,
    pub audit_log_path: String,
    pub admin_code_hash: String,
    pub audio_settings: AudioSettings,
    pub agent_settings: AgentSettings,
    pub security_settings: SecuritySettings,
    pub ui_settings: UiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
    pub max_recording_duration_ms: u64,
    pub auto_save_recordings: bool,
    pub audio_storage_path: String,
    pub whisper_language: Option<String>,
    pub enable_noise_reduction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    pub default_model: String,
    pub max_conversation_history: usize,
    pub context_window_size: usize,
    pub temperature: f32,
    pub max_tokens: u32,
    pub enable_rag: bool,
    pub rag_similarity_threshold: f32,
    pub rag_max_results: usize,
    pub auto_switch_models: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub require_admin_approval: bool,
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub enable_audit_logging: bool,
    pub emergency_stop_enabled: bool,
    pub code_modification_enabled: bool,
    pub allowed_modification_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: String,
    pub enable_floating_panel: bool,
    pub panel_transparency: f32,
    pub waveform_color: String,
    pub enable_live_waveform: bool,
    pub waveform_update_rate_ms: u32,
    pub show_audio_visualizations: bool,
    pub enable_notifications: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/dyht".to_string(),
            whisper_model_path: "./models/whisper-base.bin".to_string(),
            audit_log_path: "./logs/audit.log".to_string(),
            admin_code_hash: "default_hash_change_me".to_string(),
            audio_settings: AudioSettings::default(),
            agent_settings: AgentSettings::default(),
            security_settings: SecuritySettings::default(),
            ui_settings: UiSettings::default(),
        }
    }
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 1, // Mono
            bit_depth: 16,
            max_recording_duration_ms: 300000, // 5 minutes
            auto_save_recordings: true,
            audio_storage_path: "./audio".to_string(),
            whisper_language: Some("en".to_string()),
            enable_noise_reduction: true,
        }
    }
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            default_model: "conversational".to_string(),
            max_conversation_history: 100,
            context_window_size: 4096,
            temperature: 0.7,
            max_tokens: 2048,
            enable_rag: true,
            rag_similarity_threshold: 0.7,
            rag_max_results: 10,
            auto_switch_models: true,
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            require_admin_approval: true,
            max_failed_attempts: 5,
            lockout_duration_minutes: 30,
            enable_audit_logging: true,
            emergency_stop_enabled: true,
            code_modification_enabled: false, // Disabled by default for safety
            allowed_modification_types: vec![
                "ui_changes".to_string(),
                "configuration_updates".to_string(),
            ],
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            enable_floating_panel: true,
            panel_transparency: 0.9,
            waveform_color: "#00ff00".to_string(),
            enable_live_waveform: true,
            waveform_update_rate_ms: 50,
            show_audio_visualizations: true,
            enable_notifications: true,
        }
    }
}

impl AppConfig {
    pub async fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if tokio::fs::metadata(&config_path).await.is_ok() {
            let config_content = tokio::fs::read_to_string(&config_path).await?;
            let config: AppConfig = serde_json::from_str(&config_content)?;
            log::info!("Loaded configuration from: {:?}", config_path);
            Ok(config)
        } else {
            log::info!("Configuration file not found, using defaults");
            let default_config = Self::default();
            default_config.save().await?;
            Ok(default_config)
        }
    }

    pub async fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // Ensure the config directory exists
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let config_content = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&config_path, config_content).await?;
        
        log::info!("Saved configuration to: {:?}", config_path);
        Ok(())
    }

    pub async fn update_audio_settings(&mut self, settings: AudioSettings) -> Result<()> {
        self.audio_settings = settings;
        self.save().await?;
        log::info!("Updated audio settings");
        Ok(())
    }

    pub async fn update_agent_settings(&mut self, settings: AgentSettings) -> Result<()> {
        self.agent_settings = settings;
        self.save().await?;
        log::info!("Updated agent settings");
        Ok(())
    }

    pub async fn update_security_settings(&mut self, settings: SecuritySettings) -> Result<()> {
        self.security_settings = settings;
        self.save().await?;
        log::info!("Updated security settings");
        Ok(())
    }

    pub async fn update_ui_settings(&mut self, settings: UiSettings) -> Result<()> {
        self.ui_settings = settings;
        self.save().await?;
        log::info!("Updated UI settings");
        Ok(())
    }

    pub async fn reset_to_defaults(&mut self) -> Result<()> {
        *self = Self::default();
        self.save().await?;
        log::info!("Reset configuration to defaults");
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        // Validate database URL format
        if !self.database_url.starts_with("postgresql://") {
            return Err(anyhow::anyhow!("Invalid database URL format"));
        }

        // Validate audio settings
        if self.audio_settings.sample_rate == 0 {
            return Err(anyhow::anyhow!("Invalid sample rate"));
        }

        if self.audio_settings.channels == 0 || self.audio_settings.channels > 2 {
            return Err(anyhow::anyhow!("Invalid channel count"));
        }

        // Validate agent settings
        if self.agent_settings.temperature < 0.0 || self.agent_settings.temperature > 2.0 {
            return Err(anyhow::anyhow!("Invalid temperature value"));
        }

        if self.agent_settings.max_tokens == 0 {
            return Err(anyhow::anyhow!("Invalid max tokens"));
        }

        // Validate UI settings
        if self.ui_settings.panel_transparency < 0.0 || self.ui_settings.panel_transparency > 1.0 {
            return Err(anyhow::anyhow!("Invalid panel transparency"));
        }

        Ok(())
    }

    pub fn get_model_endpoint(&self, model_name: &str) -> Option<String> {
        // TODO: Implement model endpoint configuration
        // This would return the appropriate endpoint for different models
        match model_name {
            "mixtral" => Some("http://localhost:8000/v1/chat/completions".to_string()),
            "llama3" => Some("http://localhost:8001/v1/chat/completions".to_string()),
            _ => None,
        }
    }

    pub fn is_development_mode(&self) -> bool {
        // Check if running in development mode
        std::env::var("TAURI_DEBUG").is_ok()
    }

    pub fn get_data_directory(&self) -> Result<PathBuf> {
        let data_dir = if self.is_development_mode() {
            PathBuf::from("./dev_data")
        } else {
            // TODO: Use proper app data directory based on OS
            PathBuf::from("./data")
        };

        std::fs::create_dir_all(&data_dir)?;
        Ok(data_dir)
    }

    pub fn get_models_directory(&self) -> Result<PathBuf> {
        let models_dir = self.get_data_directory()?.join("models");
        std::fs::create_dir_all(&models_dir)?;
        Ok(models_dir)
    }

    pub fn get_audio_directory(&self) -> Result<PathBuf> {
        let audio_dir = PathBuf::from(&self.audio_settings.audio_storage_path);
        std::fs::create_dir_all(&audio_dir)?;
        Ok(audio_dir)
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = if std::env::var("TAURI_DEBUG").is_ok() {
            PathBuf::from("./dev_config")
        } else {
            // TODO: Use proper config directory based on OS
            PathBuf::from("./config")
        };

        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("dyht_config.json"))
    }

    pub fn get_log_level(&self) -> log::LevelFilter {
        if self.is_development_mode() {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        }
    }
}