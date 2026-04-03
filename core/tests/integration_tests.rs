//! Integration tests for NetSentinel Core

use netsentinel_core::rules::{Rule, Condition, ConditionField, ConditionOperator, RuleAction, ProcessContext, BehaviorFingerprint};

#[test]
fn test_rule_creation() {
    let conditions = vec![
        Condition {
            field: ConditionField::ProcessName,
            operator: ConditionOperator::Contains,
            value: "video".to_string(),
        },
        Condition {
            field: ConditionField::UploadSpeed,
            operator: ConditionOperator::GreaterThan,
            value: "500".to_string(),
        },
    ];

    let rule = Rule::new(
        "Block High Upload Video Clients",
        conditions,
        RuleAction::Block { duration_secs: 300 },
    );

    assert!(rule.enabled);
    assert_eq!(rule.conditions.len(), 2);
}

#[test]
fn test_rule_matching() {
    let conditions = vec![
        Condition {
            field: ConditionField::ProcessName,
            operator: ConditionOperator::Contains,
            value: "video".to_string(),
        },
        Condition {
            field: ConditionField::UploadSpeed,
            operator: ConditionOperator::GreaterThan,
            value: "500".to_string(),
        },
    ];

    let rule = Rule::new(
        "Test Rule",
        conditions,
        RuleAction::Block { duration_secs: 60 },
    );

    // Should match
    let matching_context = ProcessContext {
        process_id: 1234,
        process_name: "video_client.exe".to_string(),
        upload_speed_kbps: 1000,
        download_speed_kbps: 100,
        connection_count: 50,
        signature_verified: false,
    };

    assert!(rule.matches(&matching_context));

    // Should not match (upload speed too low)
    let non_matching_context = ProcessContext {
        process_id: 5678,
        process_name: "video_client.exe".to_string(),
        upload_speed_kbps: 100,
        download_speed_kbps: 1000,
        connection_count: 10,
        signature_verified: true,
    };

    assert!(!rule.matches(&non_matching_context));
}

#[test]
fn test_pcdn_behavior_detection() {
    // PCDN-like process
    let pcdn_process = ProcessContext {
        process_id: 1234,
        process_name: "pcdn_client.exe".to_string(),
        upload_speed_kbps: 5000,
        download_speed_kbps: 500,
        connection_count: 128,
        signature_verified: false,
    };

    assert!(BehaviorFingerprint::is_pcdn_like(&pcdn_process));
    assert!(!BehaviorFingerprint::is_normal_traffic(&pcdn_process));

    // Normal process
    let normal_process = ProcessContext {
        process_id: 5678,
        process_name: "browser.exe".to_string(),
        upload_speed_kbps: 500,
        download_speed_kbps: 5000,
        connection_count: 15,
        signature_verified: true,
    };

    assert!(!BehaviorFingerprint::is_pcdn_like(&normal_process));
    assert!(BehaviorFingerprint::is_normal_traffic(&normal_process));
}

#[test]
fn test_rule_action_variants() {
    let _block_temp = RuleAction::Block { duration_secs: 300 };
    let _block_perm = RuleAction::BlockPermanent;
    let _rate_limit = RuleAction::RateLimit { max_kbps: 100 };
    let _warn = RuleAction::Warn;
    let _allow = RuleAction::Allow;

    // All variants should compile and be constructible
}
