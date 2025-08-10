// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;
mod audio;
mod memory;
mod security;
mod audit;
mod config;

use tauri::{Manager, State};
use std::sync::Arc;
use tokio::sync::RwLock;

use agent::AgentCore;
use audio::AudioProcessor;
use memory::MemoryManager;
use security::SecurityManager;
use audit::AuditLogger;
use config::AppConfig;

pub type AppState = Arc<RwLock<DyhtApp>>;

#[derive(Debug)]
pub struct DyhtApp {
    pub agent_core: AgentCore,
    pub audio_processor: AudioProcessor,
    pub memory_manager: MemoryManager,
    pub security_manager: SecurityManager,
    pub audit_logger: AuditLogger,
    pub config: AppConfig,
}

impl DyhtApp {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = AppConfig::load().await?;
        let audit_logger = AuditLogger::new(&config.audit_log_path).await?;
        let memory_manager = MemoryManager::new(&config.database_url).await?;
        let security_manager = SecurityManager::new(&config.admin_code_hash);
        let audio_processor = AudioProcessor::new(&config.whisper_model_path).await?;
        let agent_core = AgentCore::new(config.clone()).await?;

        Ok(Self {
            agent_core,
            audio_processor,
            memory_manager,
            security_manager,
            audit_logger,
            config,
        })
    }
}

// Tauri commands
#[tauri::command]
async fn init_agent(app_state: State<'_, AppState>) -> Result<String, String> {
    let app = app_state.read().await;
    app.audit_logger.log_action("agent_init", "Agent initialization requested").await
        .map_err(|e| e.to_string())?;
    Ok("Agent initialized".to_string())
}

#[tauri::command]
async fn process_audio(
    app_state: State<'_, AppState>,
    audio_data: Vec<u8>
) -> Result<String, String> {
    let app = app_state.write().await;
    app.audio_processor.process_audio(audio_data).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn chat_with_agent(
    app_state: State<'_, AppState>,
    message: String
) -> Result<String, String> {
    let app = app_state.write().await;
    app.agent_core.process_message(message).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn request_code_modification(
    app_state: State<'_, AppState>,
    modification_request: String,
    admin_code: String
) -> Result<String, String> {
    let app = app_state.write().await;
    
    if !app.security_manager.verify_admin_code(&admin_code) {
        return Err("Invalid admin code".to_string());
    }

    app.security_manager.request_code_modification(modification_request).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_memory_context(
    app_state: State<'_, AppState>,
    query: String
) -> Result<Vec<String>, String> {
    let app = app_state.read().await;
    app.memory_manager.retrieve_context(&query).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn emergency_kill_switch(app_state: State<'_, AppState>) -> Result<String, String> {
    let app = app_state.write().await;
    app.audit_logger.log_action("emergency_kill", "Emergency kill switch activated").await
        .map_err(|e| e.to_string())?;
    app.agent_core.emergency_shutdown().await
        .map_err(|e| e.to_string())?;
    Ok("Emergency shutdown complete".to_string())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let app_state = Arc::new(RwLock::new(
        DyhtApp::new().await.expect("Failed to initialize DYHT app")
    ));

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            init_agent,
            process_audio,
            chat_with_agent,
            request_code_modification,
            get_memory_context,
            emergency_kill_switch
        ])
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            #[cfg(debug_assertions)]
            window.open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}