use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Critical,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub action: String,
    pub description: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug)]
pub struct AuditLogger {
    pub log_file_path: PathBuf,
    pub max_log_size_mb: u64,
    pub retention_days: u32,
}

impl AuditLogger {
    pub async fn new(log_file_path: &str) -> Result<Self> {
        let path = PathBuf::from(log_file_path);
        
        // Ensure the log directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let logger = Self {
            log_file_path: path,
            max_log_size_mb: 100, // 100MB max log file size
            retention_days: 90,   // 90 days retention
        };

        // Log the logger initialization
        logger.log_action("audit_logger_init", "Audit logger initialized").await?;
        
        Ok(logger)
    }

    pub async fn log_action(&self, action: &str, description: &str) -> Result<()> {
        self.log_entry(AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            action: action.to_string(),
            description: description.to_string(),
            user_id: None,
            session_id: None,
            ip_address: None,
            metadata: serde_json::json!({}),
        }).await
    }

    pub async fn log_security_event(
        &self,
        action: &str,
        description: &str,
        user_id: Option<String>,
        ip_address: Option<String>,
    ) -> Result<()> {
        self.log_entry(AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Security,
            action: action.to_string(),
            description: description.to_string(),
            user_id,
            session_id: None,
            ip_address,
            metadata: serde_json::json!({
                "security_event": true,
                "requires_review": true
            }),
        }).await
    }

    pub async fn log_code_modification(
        &self,
        modification_id: Uuid,
        action: &str,
        description: &str,
        user_id: Option<String>,
    ) -> Result<()> {
        self.log_entry(AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Critical,
            action: action.to_string(),
            description: description.to_string(),
            user_id,
            session_id: None,
            ip_address: None,
            metadata: serde_json::json!({
                "modification_id": modification_id,
                "code_modification": true,
                "requires_admin_review": true
            }),
        }).await
    }

    pub async fn log_agent_interaction(
        &self,
        action: &str,
        input: &str,
        output: &str,
        model_used: &str,
    ) -> Result<()> {
        self.log_entry(AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            action: action.to_string(),
            description: format!("Agent interaction: {}", action),
            user_id: None,
            session_id: None,
            ip_address: None,
            metadata: serde_json::json!({
                "input": input,
                "output": output,
                "model_used": model_used,
                "interaction_type": "agent_chat"
            }),
        }).await
    }

    pub async fn log_audio_processing(
        &self,
        action: &str,
        audio_id: Option<Uuid>,
        duration_ms: Option<u64>,
        transcription: Option<&str>,
    ) -> Result<()> {
        self.log_entry(AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            action: action.to_string(),
            description: format!("Audio processing: {}", action),
            user_id: None,
            session_id: None,
            ip_address: None,
            metadata: serde_json::json!({
                "audio_id": audio_id,
                "duration_ms": duration_ms,
                "transcription": transcription,
                "processing_type": "audio"
            }),
        }).await
    }

    pub async fn log_memory_operation(
        &self,
        action: &str,
        memory_id: Option<Uuid>,
        query: Option<&str>,
        results_count: Option<usize>,
    ) -> Result<()> {
        self.log_entry(AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            action: action.to_string(),
            description: format!("Memory operation: {}", action),
            user_id: None,
            session_id: None,
            ip_address: None,
            metadata: serde_json::json!({
                "memory_id": memory_id,
                "query": query,
                "results_count": results_count,
                "operation_type": "memory"
            }),
        }).await
    }

    pub async fn log_error(
        &self,
        action: &str,
        error_message: &str,
        error_details: Option<serde_json::Value>,
    ) -> Result<()> {
        self.log_entry(AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Error,
            action: action.to_string(),
            description: format!("Error: {}", error_message),
            user_id: None,
            session_id: None,
            ip_address: None,
            metadata: serde_json::json!({
                "error": true,
                "error_message": error_message,
                "error_details": error_details
            }),
        }).await
    }

    pub async fn log_system_event(
        &self,
        event_type: &str,
        description: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        self.log_entry(AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            action: format!("system_{}", event_type),
            description: description.to_string(),
            user_id: None,
            session_id: None,
            ip_address: None,
            metadata: metadata.unwrap_or_else(|| serde_json::json!({
                "system_event": true,
                "event_type": event_type
            })),
        }).await
    }

    async fn log_entry(&self, entry: AuditLogEntry) -> Result<()> {
        // Check if log rotation is needed
        self.check_and_rotate_logs().await?;

        // Serialize the log entry
        let log_line = format!("{}\n", serde_json::to_string(&entry)?);

        // Append to log file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
            .await?;

        file.write_all(log_line.as_bytes()).await?;
        file.flush().await?;

        // Also log to stdout for development
        println!("AUDIT: {} - {} - {}", entry.timestamp, entry.action, entry.description);

        Ok(())
    }

    async fn check_and_rotate_logs(&self) -> Result<()> {
        // Check if log file exists and its size
        if let Ok(metadata) = tokio::fs::metadata(&self.log_file_path).await {
            let size_mb = metadata.len() / (1024 * 1024);
            
            if size_mb >= self.max_log_size_mb {
                self.rotate_logs().await?;
            }
        }

        Ok(())
    }

    async fn rotate_logs(&self) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let rotated_path = self.log_file_path.with_extension(format!("log.{}", timestamp));

        // Move current log to rotated file
        tokio::fs::rename(&self.log_file_path, &rotated_path).await?;

        log::info!("Rotated audit log to: {:?}", rotated_path);

        // Clean up old log files
        self.cleanup_old_logs().await?;

        Ok(())
    }

    async fn cleanup_old_logs(&self) -> Result<()> {
        let log_dir = self.log_file_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid log file path"))?;

        let cutoff_time = Utc::now() - chrono::Duration::days(self.retention_days as i64);

        let mut entries = tokio::fs::read_dir(log_dir).await?;
        let mut cleaned_count = 0;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extension.to_string_lossy().starts_with("log.") {
                    if let Ok(metadata) = entry.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            let modified_time: DateTime<Utc> = modified.into();
                            
                            if modified_time < cutoff_time {
                                tokio::fs::remove_file(&path).await?;
                                cleaned_count += 1;
                                log::info!("Removed old log file: {:?}", path);
                            }
                        }
                    }
                }
            }
        }

        if cleaned_count > 0 {
            log::info!("Cleaned up {} old log files", cleaned_count);
        }

        Ok(())
    }

    pub async fn search_logs(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        action_filter: Option<String>,
        level_filter: Option<LogLevel>,
        limit: Option<usize>,
    ) -> Result<Vec<AuditLogEntry>> {
        // TODO: Implement efficient log searching
        // For a production system, this would:
        // 1. Read through log files in reverse chronological order
        // 2. Parse JSON lines
        // 3. Apply filters
        // 4. Return results
        
        // Placeholder implementation
        log::info!("Log search requested (not fully implemented in scaffold)");
        Ok(vec![])
    }

    pub async fn export_logs(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        export_path: &str,
    ) -> Result<()> {
        // TODO: Implement log export functionality
        // This would extract logs within the time range and export to specified format
        log::info!("Log export requested from {} to {} (not implemented in scaffold)", start_time, end_time);
        Ok(())
    }

    pub async fn get_log_statistics(&self) -> Result<serde_json::Value> {
        // TODO: Implement log statistics
        // This would return counts by level, actions, time periods, etc.
        Ok(serde_json::json!({
            "total_entries": 0,
            "by_level": {},
            "by_action": {},
            "time_range": {
                "oldest": null,
                "newest": null
            }
        }))
    }
}