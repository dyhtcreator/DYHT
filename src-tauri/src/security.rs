use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModificationStatus {
    Pending,
    Approved,
    Rejected,
    Applied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeModificationRequest {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub proposed_changes: String,
    pub status: ModificationStatus,
    pub approved_by: Option<String>,
    pub approval_timestamp: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub applied_timestamp: Option<DateTime<Utc>>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,     // UI changes, logging adjustments
    Medium,  // Algorithm modifications, new features
    High,    // Core system changes, security modifications
    Critical, // System-level changes, database modifications
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub pattern: String, // Regex pattern for code analysis
    pub risk_level: RiskLevel,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct SecurityManager {
    pub admin_code_hash: String,
    pub modification_requests: RwLock<HashMap<Uuid, CodeModificationRequest>>,
    pub security_rules: RwLock<Vec<SecurityRule>>,
    pub failed_attempts: RwLock<HashMap<String, u32>>,
    pub lockout_threshold: u32,
}

impl SecurityManager {
    pub fn new(admin_code_hash: &str) -> Self {
        let default_rules = Self::create_default_security_rules();
        
        Self {
            admin_code_hash: admin_code_hash.to_string(),
            modification_requests: RwLock::new(HashMap::new()),
            security_rules: RwLock::new(default_rules),
            failed_attempts: RwLock::new(HashMap::new()),
            lockout_threshold: 5,
        }
    }

    pub fn verify_admin_code(&self, code: &str) -> bool {
        let provided_hash = self.hash_admin_code(code);
        
        if provided_hash != self.admin_code_hash {
            // TODO: In a real implementation, track failed attempts by IP/session
            log::warn!("Failed admin code verification attempt");
            return false;
        }
        
        true
    }

    pub async fn request_code_modification(&self, description: String) -> Result<String> {
        let modification_id = Uuid::new_v4();
        
        // TODO: Implement actual code analysis
        let proposed_changes = format!(
            "Placeholder code changes for: {}\n\n// Analysis pending...\n// Risk assessment pending...",
            description
        );
        
        let risk_level = self.assess_risk_level(&description, &proposed_changes).await;
        
        let request = CodeModificationRequest {
            id: modification_id,
            timestamp: Utc::now(),
            description: description.clone(),
            proposed_changes,
            status: ModificationStatus::Pending,
            approved_by: None,
            approval_timestamp: None,
            rejection_reason: None,
            applied_timestamp: None,
            risk_level,
        };

        self.modification_requests.write().await.insert(modification_id, request);
        
        log::info!("Code modification request created: {} - {}", modification_id, description);
        
        Ok(format!(
            "Code modification request submitted with ID: {}. Risk level: {:?}. Awaiting admin approval.",
            modification_id, risk_level
        ))
    }

    pub async fn approve_modification(
        &self,
        modification_id: Uuid,
        admin_code: &str,
        approver_id: String,
    ) -> Result<()> {
        if !self.verify_admin_code(admin_code) {
            return Err(anyhow::anyhow!("Invalid admin code"));
        }

        let mut requests = self.modification_requests.write().await;
        
        if let Some(request) = requests.get_mut(&modification_id) {
            if !matches!(request.status, ModificationStatus::Pending) {
                return Err(anyhow::anyhow!("Modification request is not pending"));
            }

            request.status = ModificationStatus::Approved;
            request.approved_by = Some(approver_id);
            request.approval_timestamp = Some(Utc::now());
            
            log::info!("Code modification approved: {}", modification_id);
            
            // TODO: Implement actual code application
            // This would involve:
            // 1. Validating the proposed changes
            // 2. Running security checks
            // 3. Applying the changes safely
            // 4. Rolling back if issues occur
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Modification request not found"))
        }
    }

    pub async fn reject_modification(
        &self,
        modification_id: Uuid,
        admin_code: &str,
        reason: String,
    ) -> Result<()> {
        if !self.verify_admin_code(admin_code) {
            return Err(anyhow::anyhow!("Invalid admin code"));
        }

        let mut requests = self.modification_requests.write().await;
        
        if let Some(request) = requests.get_mut(&modification_id) {
            if !matches!(request.status, ModificationStatus::Pending) {
                return Err(anyhow::anyhow!("Modification request is not pending"));
            }

            request.status = ModificationStatus::Rejected;
            request.rejection_reason = Some(reason);
            
            log::info!("Code modification rejected: {}", modification_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Modification request not found"))
        }
    }

    pub async fn get_pending_modifications(&self) -> Vec<CodeModificationRequest> {
        self.modification_requests
            .read()
            .await
            .values()
            .filter(|req| matches!(req.status, ModificationStatus::Pending))
            .cloned()
            .collect()
    }

    pub async fn get_modification_history(&self) -> Vec<CodeModificationRequest> {
        let mut history: Vec<_> = self.modification_requests.read().await.values().cloned().collect();
        history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        history
    }

    async fn assess_risk_level(&self, description: &str, code: &str) -> RiskLevel {
        let high_risk_patterns = [
            r"(?i)(database|sql|delete|drop|truncate)",
            r"(?i)(system|exec|command|shell)",
            r"(?i)(file|write|delete|remove)",
            r"(?i)(security|auth|password|token)",
            r"(?i)(network|socket|connection)",
        ];

        let medium_risk_patterns = [
            r"(?i)(algorithm|model|inference)",
            r"(?i)(memory|cache|store)",
            r"(?i)(config|setting|parameter)",
        ];

        // Check against high risk patterns
        for pattern in &high_risk_patterns {
            let regex = regex::Regex::new(pattern).unwrap();
            if regex.is_match(description) || regex.is_match(code) {
                return RiskLevel::High;
            }
        }

        // Check against medium risk patterns
        for pattern in &medium_risk_patterns {
            let regex = regex::Regex::new(pattern).unwrap();
            if regex.is_match(description) || regex.is_match(code) {
                return RiskLevel::Medium;
            }
        }

        RiskLevel::Low
    }

    pub async fn add_security_rule(&self, rule: SecurityRule) -> Result<()> {
        let mut rules = self.security_rules.write().await;
        
        // Check if rule with same name already exists
        if rules.iter().any(|r| r.name == rule.name) {
            return Err(anyhow::anyhow!("Security rule with name '{}' already exists", rule.name));
        }
        
        rules.push(rule);
        log::info!("Added new security rule");
        Ok(())
    }

    pub async fn update_admin_code(&self, old_code: &str, new_code: &str) -> Result<()> {
        if !self.verify_admin_code(old_code) {
            return Err(anyhow::anyhow!("Invalid current admin code"));
        }

        // TODO: In a real implementation, this would update the stored hash
        // For now, just log the action
        log::info!("Admin code update requested (not implemented in scaffold)");
        Ok(())
    }

    pub async fn enable_emergency_lockdown(&self) -> Result<()> {
        // TODO: Implement emergency lockdown
        // This would:
        // 1. Disable all pending modifications
        // 2. Block new modification requests
        // 3. Alert administrators
        // 4. Log the emergency action
        
        log::warn!("Emergency lockdown activated");
        Ok(())
    }

    fn hash_admin_code(&self, code: &str) -> String {
        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn create_default_security_rules() -> Vec<SecurityRule> {
        vec![
            SecurityRule {
                id: Uuid::new_v4(),
                name: "Database Access".to_string(),
                description: "Detect database operations that could affect data integrity".to_string(),
                pattern: r"(?i)(DROP|DELETE|TRUNCATE|ALTER)\s+(TABLE|DATABASE)".to_string(),
                risk_level: RiskLevel::Critical,
                enabled: true,
            },
            SecurityRule {
                id: Uuid::new_v4(),
                name: "System Commands".to_string(),
                description: "Detect system command execution attempts".to_string(),
                pattern: r"(?i)(system|exec|spawn|command)".to_string(),
                risk_level: RiskLevel::High,
                enabled: true,
            },
            SecurityRule {
                id: Uuid::new_v4(),
                name: "File Operations".to_string(),
                description: "Monitor file system modifications".to_string(),
                pattern: r"(?i)(write_file|delete_file|remove_file)".to_string(),
                risk_level: RiskLevel::Medium,
                enabled: true,
            },
            SecurityRule {
                id: Uuid::new_v4(),
                name: "Network Access".to_string(),
                description: "Monitor network connection attempts".to_string(),
                pattern: r"(?i)(connect|socket|http|https|tcp|udp)".to_string(),
                risk_level: RiskLevel::Medium,
                enabled: true,
            },
        ]
    }
}