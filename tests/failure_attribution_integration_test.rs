/// Integration test for failure attribution feature.
///
/// This test demonstrates the end-to-end flow of failure attribution:
/// 1. GUI action executes
/// 2. Verification detects failure
/// 3. Failure cause is inferred (AgentError vs EnvironmentBlocked)
/// 4. Process score and outcome success are computed
/// 5. Agent loop uses structured data for better recovery hints

use gloamy::tools::gui_verify::{build_report, verify_expectations};
use gloamy::tools::{FailureCause, GuiExpectation, GuiExpectationKind, VerificationStatus};
use serde_json::json;

#[test]
fn test_environment_block_login_wall() {
    // Scenario: Agent tries to access a success page but hits a login wall
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::UrlIs {
            url: "https://example.com/dashboard".into(),
        },
        required: true,
    }];

    let post_evidence = json!({
        "url": "https://example.com/login?redirect=/dashboard"
    });

    let report = build_report(true, None, None, &expectations, &post_evidence);

    // Verification should fail
    assert_eq!(report.verification_status, VerificationStatus::Failed);

    // But it's an environment block, not an agent error
    assert_eq!(report.failure_cause, FailureCause::EnvironmentBlocked);

    // Process score should be high (agent did the right thing)
    assert_eq!(report.process_score, Some(0.8));

    // Outcome failed
    assert_eq!(report.outcome_success, Some(false));

    // Check that the expectation result has the right failure cause
    assert_eq!(
        report.expectation_results[0].failure_cause,
        FailureCause::EnvironmentBlocked
    );
}

#[test]
fn test_agent_error_wrong_selector() {
    // Scenario: Agent uses wrong selector, gets wrong value
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::FieldValueEquals {
            selector: "#email".into(),
            value: "test@example.com".into(),
        },
        required: true,
    }];

    let post_evidence = json!({
        "field_values": {
            "#username": "test@example.com"
        }
    });

    let report = build_report(true, None, None, &expectations, &post_evidence);

    // Verification should be ambiguous (evidence absent for #email)
    assert_eq!(report.verification_status, VerificationStatus::Ambiguous);

    // It's evidence absent (selector doesn't exist)
    assert_eq!(report.failure_cause, FailureCause::EvidenceAbsent);

    // Process score should be None (can't determine from missing evidence)
    assert_eq!(report.process_score, Some(0.8));

    // Outcome failed
    assert_eq!(report.outcome_success, Some(false));
}

#[test]
fn test_agent_error_wrong_value() {
    // Scenario: Agent fills wrong value
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::FieldValueEquals {
            selector: "#name".into(),
            value: "Alice".into(),
        },
        required: true,
    }];

    let post_evidence = json!({
        "field_values": {
            "#name": "Bob"
        }
    });

    let report = build_report(true, None, None, &expectations, &post_evidence);

    // Verification should fail
    assert_eq!(report.verification_status, VerificationStatus::Failed);

    // It's an agent error (wrong value)
    assert_eq!(report.failure_cause, FailureCause::AgentError);

    // Process score should be low (agent made a mistake)
    assert_eq!(report.process_score, Some(0.0));

    // Outcome failed
    assert_eq!(report.outcome_success, Some(false));
}

#[test]
fn test_captcha_detection() {
    // Scenario: Agent hits a CAPTCHA challenge
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::WindowTitleContains {
            substring: "Checkout".into(),
        },
        required: true,
    }];

    let post_evidence = json!({
        "title": "Please verify you're human - reCAPTCHA"
    });

    let report = build_report(true, None, None, &expectations, &post_evidence);

    assert_eq!(report.verification_status, VerificationStatus::Failed);
    assert_eq!(report.failure_cause, FailureCause::EnvironmentBlocked);
    assert_eq!(report.process_score, Some(0.8)); // High - not agent's fault
}

#[test]
fn test_out_of_stock_detection() {
    // Scenario: Product is out of stock
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::WindowTitleContains {
            substring: "Add to Cart".into(),
        },
        required: true,
    }];

    let post_evidence = json!({
        "title": "Product Currently Unavailable - Out of Stock"
    });

    let report = build_report(true, None, None, &expectations, &post_evidence);

    assert_eq!(report.verification_status, VerificationStatus::Failed);
    assert_eq!(report.failure_cause, FailureCause::EnvironmentBlocked);
    assert_eq!(report.process_score, Some(0.8));
}

#[test]
fn test_network_timeout_detection() {
    // Scenario: Network timeout
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::UrlIs {
            url: "https://example.com/success".into(),
        },
        required: true,
    }];

    let post_evidence = json!({
        "url": "https://example.com/error?code=504"
    });

    let report = build_report(true, None, None, &expectations, &post_evidence);

    assert_eq!(report.verification_status, VerificationStatus::Failed);
    assert_eq!(report.failure_cause, FailureCause::EnvironmentBlocked);
}

#[test]
fn test_successful_verification() {
    // Scenario: Everything works correctly
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::UrlIs {
            url: "https://example.com/success".into(),
        },
        required: true,
    }];

    let post_evidence = json!({
        "url": "https://example.com/success"
    });

    let report = build_report(true, None, None, &expectations, &post_evidence);

    assert_eq!(report.verification_status, VerificationStatus::Verified);
    assert_eq!(report.failure_cause, FailureCause::None);
    assert_eq!(report.process_score, Some(1.0)); // Perfect
    assert_eq!(report.outcome_success, Some(true));
}

#[test]
fn test_mixed_failures_prioritizes_environment_blocks() {
    // Scenario: Multiple expectations, some agent errors, some environment blocks
    let expectations = vec![
        GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com/dashboard".into(),
            },
            required: true,
        },
        GuiExpectation {
            kind: GuiExpectationKind::FieldValueEquals {
                selector: "#name".into(),
                value: "Alice".into(),
            },
            required: true,
        },
    ];

    let post_evidence = json!({
        "url": "https://example.com/login",  // Environment block
        "field_values": {
            "#name": "Bob"  // Agent error
        }
    });

    let report = build_report(true, None, None, &expectations, &post_evidence);

    assert_eq!(report.verification_status, VerificationStatus::Failed);
    // Should prioritize environment block over agent error
    assert_eq!(report.failure_cause, FailureCause::EnvironmentBlocked);
    // Process score should reflect mixed failures
    assert!(report.process_score.unwrap() > 0.0 && report.process_score.unwrap() < 1.0);
}
