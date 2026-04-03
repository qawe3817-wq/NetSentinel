//! Rule Engine Module
//! 
//! Defines network filtering rules with condition blocks

use serde::{Deserialize, Serialize};

/// Network filtering rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub conditions: Vec<Condition>,
    pub action: RuleAction,
    pub priority: u32,
}

/// Condition block for rule matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: ConditionField,
    pub operator: ConditionOperator,
    pub value: String,
}

/// Available condition fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionField {
    ProcessName,
    UploadSpeed,
    DownloadSpeed,
    ConnectionCount,
    UploadDownloadRatio,
    TargetIp,
    SignatureStatus,
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionOperator {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
    Regex,
}

/// Rule actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    /// Block connection for specified duration (seconds)
    Block { duration_secs: u64 },
    /// Permanent block
    BlockPermanent,
    /// Rate limit to specified KB/s
    RateLimit { max_kbps: u32 },
    /// Only log and warn
    Warn,
    /// Allow without inspection
    Allow,
}

impl Rule {
    /// Create a new rule from condition blocks
    pub fn new(
        name: impl Into<String>,
        conditions: Vec<Condition>,
        action: RuleAction,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            enabled: true,
            conditions,
            action,
            priority: 100,
        }
    }

    /// Check if a process matches this rule's conditions
    pub fn matches(&self, context: &ProcessContext) -> bool {
        self.conditions.iter().all(|cond| cond.evaluate(context))
    }
}

impl Condition {
    /// Evaluate condition against process context
    pub fn evaluate(&self, context: &ProcessContext) -> bool {
        match &self.field {
            ConditionField::ProcessName => {
                let name = &context.process_name;
                match self.operator {
                    ConditionOperator::Contains => name.contains(&self.value),
                    ConditionOperator::Equals => name == &self.value,
                    ConditionOperator::Regex => {
                        // In production: use regex crate
                        name.contains(&self.value)
                    }
                    _ => false,
                }
            }
            ConditionField::UploadSpeed => {
                let speed = context.upload_speed_kbps;
                let threshold: u32 = self.value.parse().unwrap_or(0);
                match self.operator {
                    ConditionOperator::GreaterThan => speed > threshold,
                    ConditionOperator::LessThan => speed < threshold,
                    ConditionOperator::Equals => speed == threshold,
                    _ => false,
                }
            }
            ConditionField::ConnectionCount => {
                let count = context.connection_count;
                let threshold: u32 = self.value.parse().unwrap_or(0);
                match self.operator {
                    ConditionOperator::GreaterThan => count > threshold,
                    ConditionOperator::LessThan => count < threshold,
                    ConditionOperator::Equals => count == threshold,
                    _ => false,
                }
            }
            // Additional field evaluations...
            _ => false,
        }
    }
}

/// Process context for rule evaluation
#[derive(Debug, Clone, Default)]
pub struct ProcessContext {
    pub process_id: u32,
    pub process_name: String,
    pub upload_speed_kbps: u32,
    pub download_speed_kbps: u32,
    pub connection_count: u32,
    pub signature_verified: bool,
}

/// PCDN behavior fingerprint detector
pub struct BehaviorFingerprint;

impl BehaviorFingerprint {
    /// Detect PCDN-like behavior patterns
    /// 
    /// PCDN characteristics:
    /// - Long-lived connections
    /// - High concurrency
    /// - Upload/Download ratio > 5:1
    /// - Dispersed target IPs
    pub fn is_pcdn_like(context: &ProcessContext) -> bool {
        let ratio = if context.download_speed_kbps > 0 {
            context.upload_speed_kbps as f32 / context.download_speed_kbps as f32
        } else {
            f32::INFINITY
        };

        // PCDN detection heuristics
        ratio > 5.0 && context.connection_count > 50
    }

    /// Normal traffic characteristics:
    /// - Burst traffic
    /// - Low concurrency
    /// - Concentrated target IPs (CDN)
    pub fn is_normal_traffic(context: &ProcessContext) -> bool {
        let ratio = if context.download_speed_kbps > 0 {
            context.upload_speed_kbps as f32 / context.download_speed_kbps as f32
        } else {
            0.0
        };

        ratio < 2.0 && context.connection_count < 20
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcdn_detection() {
        let pc = ProcessContext {
            process_id: 1234,
            process_name: "video_client.exe".to_string(),
            upload_speed_kbps: 5000,
            download_speed_kbps: 500,
            connection_count: 128,
            signature_verified: false,
        };

        assert!(BehaviorFingerprint::is_pcdn_like(&pc));
        assert!(!BehaviorFingerprint::is_normal_traffic(&pc));
    }
}
