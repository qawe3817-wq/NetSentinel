//! Rule Engine Module
//! 
//! Defines network filtering rules with condition-based matching.
//! 
//! # Overview
//! 
//! The rule engine provides a flexible system for defining network filtering policies.
//! Each rule consists of:
//! - A set of conditions that must all match (AND logic)
//! - An action to take when conditions are satisfied
//! - A priority for rule ordering
//! 
//! # Example
//! 
//! ```rust
//! use netsentinel_core::rules::{Rule, Condition, ConditionField, ConditionOperator, RuleAction};
//! 
//! // Create a rule to block high-upload processes
//! let rule = Rule::new(
//!     "Block PCDN",
//!     vec![
//!         Condition {
//!             field: ConditionField::UploadSpeed,
//!             operator: ConditionOperator::GreaterThan,
//!             value: "1024".to_string(), // 1 MB/s
//!         },
//!         Condition {
//!             field: ConditionField::ConnectionCount,
//!             operator: ConditionOperator::GreaterThan,
//!             value: "50".to_string(),
//!         },
//!     ],
//!     RuleAction::block_with_duration(600), // Block for 10 minutes
//! );
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Rule evaluation errors
#[derive(Debug, Error)]
pub enum RuleError {
    #[error("Invalid condition value: {0}")]
    InvalidConditionValue(String),
    
    #[error("Rule must have at least one condition")]
    EmptyConditions,
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Network filtering rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier for the rule
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Whether the rule is currently active
    pub enabled: bool,
    /// Conditions that must all match for the rule to apply
    pub conditions: Vec<Condition>,
    /// Action to take when rule matches
    pub action: RuleAction,
    /// Priority (lower number = higher priority)
    pub priority: u32,
}

/// Condition block for rule matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Field to evaluate against
    pub field: ConditionField,
    /// Comparison operator
    pub operator: ConditionOperator,
    /// Value to compare against
    pub value: String,
}

/// Available condition fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionField {
    /// Process executable name
    ProcessName,
    /// Upload speed in KB/s
    UploadSpeed,
    /// Download speed in KB/s
    DownloadSpeed,
    /// Number of active connections
    ConnectionCount,
    /// Ratio of upload to download
    UploadDownloadRatio,
    /// Target IP address
    TargetIp,
    /// Whether process signature is verified
    SignatureStatus,
}

/// Condition operators for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionOperator {
    /// Exact equality match
    Equals,
    /// Substring contains match
    Contains,
    /// Greater than comparison
    GreaterThan,
    /// Less than comparison
    LessThan,
    /// Regular expression match
    Regex,
}

/// Rule actions to take when conditions match
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    /// Block connection for specified duration (seconds)
    #[serde(rename = "block")]
    Block { 
        #[serde(rename = "duration_secs")]
        duration_secs: u64 
    },
    /// Permanent block until manually removed
    #[serde(rename = "block_permanent")]
    BlockPermanent,
    /// Rate limit to specified KB/s
    #[serde(rename = "rate_limit")]
    RateLimit { 
        #[serde(rename = "max_kbps")]
        max_kbps: u32 
    },
    /// Only log and warn without blocking
    #[serde(rename = "warn")]
    Warn,
    /// Allow without inspection
    #[serde(rename = "allow")]
    Allow,
}

impl Rule {
    /// Create a new rule from condition blocks
    /// 
    /// # Arguments
    /// 
    /// * `name` - Human-readable name for the rule
    /// * `conditions` - List of conditions that must all match (AND logic)
    /// * `action` - Action to take when conditions are satisfied
    /// 
    /// # Returns
    /// 
    /// A new `Rule` with auto-generated UUID and default priority
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

    /// Create a new rule with validation
    /// 
    /// # Errors
    /// 
    /// Returns `RuleError::EmptyConditions` if no conditions provided
    pub fn try_new(
        name: impl Into<String>,
        conditions: Vec<Condition>,
        action: RuleAction,
    ) -> Result<Self, RuleError> {
        if conditions.is_empty() {
            return Err(RuleError::EmptyConditions);
        }

        Ok(Self::new(name, conditions, action))
    }

    /// Check if a process matches this rule's conditions
    /// 
    /// All conditions must match (AND logic) for the rule to apply
    pub fn matches(&self, context: &ProcessContext) -> bool {
        if !self.enabled || self.conditions.is_empty() {
            return false;
        }
        
        self.conditions.iter().all(|cond| cond.evaluate(context))
    }

    /// Helper to create a block action with duration in seconds
    pub fn block_with_duration(secs: u64) -> RuleAction {
        RuleAction::Block { duration_secs: secs }
    }

    /// Helper to create a block action with duration in minutes
    pub fn block_with_minutes(mins: u64) -> RuleAction {
        RuleAction::Block { duration_secs: mins * 60 }
    }

    /// Helper to create a rate limit action
    pub fn rate_limit_kbps(kbps: u32) -> RuleAction {
        RuleAction::RateLimit { max_kbps: kbps }
    }
}

impl Condition {
    /// Evaluate condition against process context
    /// 
    /// # Panics
    /// 
    /// May panic if value cannot be parsed as numeric for numeric operators
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
            ConditionField::DownloadSpeed => {
                let speed = context.download_speed_kbps;
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
            ConditionField::UploadDownloadRatio => {
                if context.download_speed_kbps == 0 {
                    // Infinite ratio - only matches GreaterThan
                    matches!(self.operator, ConditionOperator::GreaterThan)
                } else {
                    let ratio = context.upload_speed_kbps as f64 
                        / context.download_speed_kbps as f64;
                    let threshold: f64 = self.value.parse().unwrap_or(0.0);
                    match self.operator {
                        ConditionOperator::GreaterThan => ratio > threshold,
                        ConditionOperator::LessThan => ratio < threshold,
                        ConditionOperator::Equals => (ratio - threshold).abs() < 0.01,
                        _ => false,
                    }
                }
            }
            ConditionField::SignatureStatus => {
                let verified = context.signature_verified;
                let expected: bool = self.value.parse().unwrap_or(false);
                match self.operator {
                    ConditionOperator::Equals => verified == expected,
                    ConditionOperator::NotVerified => !verified,
                    _ => false,
                }
            }
            ConditionField::TargetIp => {
                // In production: implement IP matching with CIDR support
                context.target_ips.iter().any(|ip| {
                    match self.operator {
                        ConditionOperator::Equals => ip == &self.value,
                        ConditionOperator::Contains => ip.contains(&self.value),
                        _ => false,
                    }
                })
            }
        }
    }
}

/// Process context for rule evaluation
#[derive(Debug, Clone, Default)]
pub struct ProcessContext {
    /// Process ID
    pub process_id: u32,
    /// Process executable name
    pub process_name: String,
    /// Current upload speed in KB/s
    pub upload_speed_kbps: u32,
    /// Current download speed in KB/s
    pub download_speed_kbps: u32,
    /// Number of active network connections
    pub connection_count: u32,
    /// Whether the process has a verified digital signature
    pub signature_verified: bool,
    /// Target IP addresses the process is communicating with
    pub target_ips: Vec<String>,
}

impl ProcessContext {
    /// Create a new process context
    pub fn new(process_id: u32, process_name: impl Into<String>) -> Self {
        Self {
            process_id,
            process_name: process_name.into(),
            ..Default::default()
        }
    }

    /// Set upload speed and return self for method chaining
    pub fn with_upload_speed(mut self, kbps: u32) -> Self {
        self.upload_speed_kbps = kbps;
        self
    }

    /// Set download speed and return self for method chaining
    pub fn with_download_speed(mut self, kbps: u32) -> Self {
        self.download_speed_kbps = kbps;
        self
    }

    /// Set connection count and return self for method chaining
    pub fn with_connection_count(mut self, count: u32) -> Self {
        self.connection_count = count;
        self
    }

    /// Calculate upload/download ratio
    /// Returns f64::INFINITY if download speed is 0
    pub fn upload_download_ratio(&self) -> f64 {
        if self.download_speed_kbps == 0 {
            f64::INFINITY
        } else {
            self.upload_speed_kbps as f64 / self.download_speed_kbps as f64
        }
    }

    /// Check if this process exhibits PCDN-like behavior
    /// 
    /// PCDN characteristics:
    /// - High upload/download ratio (> 5:1)
    /// - Many concurrent connections (> 50)
    pub fn is_pcdn_like(&self) -> bool {
        self.upload_download_ratio() > 5.0 && self.connection_count > 50
    }

    /// Check if this process exhibits normal traffic patterns
    /// 
    /// Normal traffic characteristics:
    /// - Balanced or download-heavy ratio (< 2:1)
    /// - Moderate connections (< 20)
    pub fn is_normal_traffic(&self) -> bool {
        self.upload_download_ratio() < 2.0 && self.connection_count < 20
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcdn_detection() {
        let pc = ProcessContext::new(1234, "video_client.exe")
            .with_upload_speed(5000)
            .with_download_speed(500)
            .with_connection_count(128);

        assert!(pc.is_pcdn_like());
        assert!(!pc.is_normal_traffic());
    }

    #[test]
    fn test_normal_traffic_detection() {
        let pc = ProcessContext::new(5678, "browser.exe")
            .with_upload_speed(100)
            .with_download_speed(500)
            .with_connection_count(10);

        assert!(!pc.is_pcdn_like());
        assert!(pc.is_normal_traffic());
    }

    #[test]
    fn test_rule_matching() {
        let rule = Rule::new(
            "Block high upload",
            vec![
                Condition {
                    field: ConditionField::UploadSpeed,
                    operator: ConditionOperator::GreaterThan,
                    value: "1000".to_string(),
                },
            ],
            RuleAction::Block { duration_secs: 60 },
        );

        let matching_ctx = ProcessContext::new(1, "test.exe")
            .with_upload_speed(2000);
        
        let non_matching_ctx = ProcessContext::new(2, "test2.exe")
            .with_upload_speed(500);

        assert!(rule.matches(&matching_ctx));
        assert!(!rule.matches(&non_matching_ctx));
    }

    #[test]
    fn test_rule_validation() {
        // Valid rule
        let result = Rule::try_new(
            "Test Rule",
            vec![Condition {
                field: ConditionField::ProcessName,
                operator: ConditionOperator::Equals,
                value: "test.exe".to_string(),
            }],
            RuleAction::Allow,
        );
        assert!(result.is_ok());

        // Invalid rule - empty conditions
        let result = Rule::try_new(
            "Invalid Rule",
            vec![],
            RuleAction::Allow,
        );
        assert!(matches!(result, Err(RuleError::EmptyConditions)));
    }

    #[test]
    fn test_helper_methods() {
        let block_action = Rule::block_with_duration(300);
        assert!(matches!(block_action, RuleAction::Block { duration_secs: 300 }));

        let block_min_action = Rule::block_with_minutes(5);
        assert!(matches!(block_min_action, RuleAction::Block { duration_secs: 300 }));

        let limit_action = Rule::rate_limit_kbps(1024);
        assert!(matches!(limit_action, RuleAction::RateLimit { max_kbps: 1024 }));
    }
}
