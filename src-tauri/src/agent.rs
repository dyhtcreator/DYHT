use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

use crate::config::AppConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Reasoning,      // Mixtral for complex reasoning
    Conversational, // Llama 3 for natural conversation
    Custom(String), // For future model swapping
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_type: ModelType,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub model_name: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub role: String,
    pub content: String,
    pub model_used: ModelType,
    pub context_retrieved: bool,
}

#[derive(Debug)]
pub struct AgentCore {
    pub config: AppConfig,
    pub models: HashMap<ModelType, ModelConfig>,
    pub conversation_history: RwLock<Vec<AgentMessage>>,
    pub active_model: RwLock<ModelType>,
    pub is_active: RwLock<bool>,
}

impl AgentCore {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let mut models = HashMap::new();
        
        // Initialize default model configurations
        models.insert(
            ModelType::Reasoning,
            ModelConfig {
                model_type: ModelType::Reasoning,
                endpoint: None, // Placeholder for future Mixtral integration
                api_key: None,
                model_name: "mixtral-8x7b-instruct".to_string(),
                max_tokens: 4096,
                temperature: 0.7,
            }
        );

        models.insert(
            ModelType::Conversational,
            ModelConfig {
                model_type: ModelType::Conversational,
                endpoint: None, // Placeholder for future Llama 3 integration
                api_key: None,
                model_name: "llama-3-8b-chat".to_string(),
                max_tokens: 2048,
                temperature: 0.9,
            }
        );

        Ok(Self {
            config,
            models,
            conversation_history: RwLock::new(Vec::new()),
            active_model: RwLock::new(ModelType::Conversational),
            is_active: RwLock::new(true),
        })
    }

    pub async fn process_message(&self, message: String) -> Result<String> {
        let is_active = *self.is_active.read().await;
        if !is_active {
            return Ok("Agent is currently inactive due to emergency shutdown.".to_string());
        }

        let active_model = self.active_model.read().await.clone();
        let model_config = self.models.get(&active_model)
            .ok_or_else(|| anyhow::anyhow!("Active model configuration not found"))?;

        // Create message record
        let agent_message = AgentMessage {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            role: "user".to_string(),
            content: message.clone(),
            model_used: active_model.clone(),
            context_retrieved: false, // TODO: Implement RAG context retrieval
        };

        // Add to conversation history
        self.conversation_history.write().await.push(agent_message);

        // TODO: Implement actual model inference
        // For now, return a placeholder response
        let response = format!(
            "Dwight AI (using {:?}): I received your message: '{}'. This is a placeholder response until the full AI integration is implemented.",
            active_model,
            message
        );

        // Record response
        let response_message = AgentMessage {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            role: "assistant".to_string(),
            content: response.clone(),
            model_used: active_model,
            context_retrieved: false,
        };

        self.conversation_history.write().await.push(response_message);

        Ok(response)
    }

    pub async fn switch_model(&self, model_type: ModelType) -> Result<()> {
        if self.models.contains_key(&model_type) {
            *self.active_model.write().await = model_type;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Model type not configured: {:?}", model_type))
        }
    }

    pub async fn add_custom_model(&self, name: String, config: ModelConfig) -> Result<()> {
        // TODO: Implement model registration and validation
        // This would allow for dynamic model swapping
        log::info!("Custom model registration requested: {}", name);
        Ok(())
    }

    pub async fn get_conversation_history(&self) -> Vec<AgentMessage> {
        self.conversation_history.read().await.clone()
    }

    pub async fn clear_conversation_history(&self) -> Result<()> {
        self.conversation_history.write().await.clear();
        Ok(())
    }

    pub async fn emergency_shutdown(&self) -> Result<()> {
        *self.is_active.write().await = false;
        log::warn!("Agent core emergency shutdown activated");
        Ok(())
    }

    pub async fn reactivate(&self) -> Result<()> {
        *self.is_active.write().await = true;
        log::info!("Agent core reactivated");
        Ok(())
    }

    pub async fn is_active(&self) -> bool {
        *self.is_active.read().await
    }

    pub async fn get_active_model(&self) -> ModelType {
        self.active_model.read().await.clone()
    }

    pub async fn analyze_for_reasoning(&self, query: String) -> Result<bool> {
        // TODO: Implement logic to determine if a query requires reasoning model
        // For now, use simple heuristics
        let reasoning_keywords = ["analyze", "compare", "calculate", "reason", "solve", "explain why"];
        let needs_reasoning = reasoning_keywords.iter()
            .any(|keyword| query.to_lowercase().contains(keyword));
        
        if needs_reasoning {
            self.switch_model(ModelType::Reasoning).await?;
        } else {
            self.switch_model(ModelType::Conversational).await?;
        }

        Ok(needs_reasoning)
    }
}