/// Demo: Failure Attribution in Action
///
/// This example demonstrates how the failure attribution system distinguishes
/// between agent errors and environment blocks, providing better recovery hints.

use gloamy::tools::gui_verify::build_report;
use gloamy::tools::{FailureCause, GuiExpectation, GuiExpectationKind};
use serde_json::json;

fn main() {
    println!("🔍 Failure Attribution Demo\n");
    println!("{}", "=".repeat(80));
    println!();

    // Scenario 1: Login Wall (Environment Block)
    println!("📋 Scenario 1: Login Wall (Environment Block)");
    println!("{}", "-".repeat(80));
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::UrlIs {
            url: "https://example.com/dashboard".into(),
        },
        required: true,
    }];
    let evidence = json!({"url": "https://example.com/login?redirect=/dashboard"});
    let report = build_report(true, None, None, &expectations, &evidence);
    
    println!("Expected: https://example.com/dashboard");
    println!("Actual:   https://example.com/login?redirect=/dashboard");
    println!();
    println!("✅ Verification Status: {:?}", report.verification_status);
    println!("🎯 Failure Cause: {:?}", report.failure_cause);
    println!("📊 Process Score: {:.2} (agent did the right thing!)", report.process_score.unwrap());
    println!("🎬 Outcome Success: {}", report.outcome_success.unwrap());
    println!();
    println!("💡 Recovery Hint: Do NOT retry - environment blocked by login wall.");
    println!("   Take screenshot, handle authentication, or report to user.");
    println!();
    println!();

    // Scenario 2: Wrong Selector (Agent Error)
    println!("📋 Scenario 2: Wrong Selector (Agent Error)");
    println!("{}", "-".repeat(80));
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::FieldValueEquals {
            selector: "#email".into(),
            value: "test@example.com".into(),
        },
        required: true,
    }];
    let evidence = json!({"field_values": {"#username": "test@example.com"}});
    let report = build_report(true, None, None, &expectations, &evidence);
    
    println!("Expected: #email = 'test@example.com'");
    println!("Actual:   #email not found (only #username exists)");
    println!();
    println!("✅ Verification Status: {:?}", report.verification_status);
    println!("🎯 Failure Cause: {:?}", report.failure_cause);
    println!("📊 Process Score: {:.2} (evidence missing)", report.process_score.unwrap());
    println!("🎬 Outcome Success: {}", report.outcome_success.unwrap());
    println!();
    println!("💡 Recovery Hint: Evidence absent - use pre_observe:auto or check selector.");
    println!();
    println!();

    // Scenario 3: Wrong Value (Agent Error)
    println!("📋 Scenario 3: Wrong Value (Agent Error)");
    println!("{}", "-".repeat(80));
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::FieldValueEquals {
            selector: "#name".into(),
            value: "Alice".into(),
        },
        required: true,
    }];
    let evidence = json!({"field_values": {"#name": "Bob"}});
    let report = build_report(true, None, None, &expectations, &evidence);
    
    println!("Expected: #name = 'Alice'");
    println!("Actual:   #name = 'Bob'");
    println!();
    println!("✅ Verification Status: {:?}", report.verification_status);
    println!("🎯 Failure Cause: {:?}", report.failure_cause);
    println!("📊 Process Score: {:.2} (agent made a mistake!)", report.process_score.unwrap());
    println!("🎬 Outcome Success: {}", report.outcome_success.unwrap());
    println!();
    println!("💡 Recovery Hint: Agent error - inspect expectation_results and retry with");
    println!("   corrected parameters. This is YOUR mistake, not the environment.");
    println!();
    println!();

    // Scenario 4: CAPTCHA Challenge (Environment Block)
    println!("📋 Scenario 4: CAPTCHA Challenge (Environment Block)");
    println!("{}", "-".repeat(80));
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::WindowTitleContains {
            substring: "Checkout".into(),
        },
        required: true,
    }];
    let evidence = json!({"title": "Please verify you're human - reCAPTCHA"});
    let report = build_report(true, None, None, &expectations, &evidence);
    
    println!("Expected: Window title contains 'Checkout'");
    println!("Actual:   'Please verify you're human - reCAPTCHA'");
    println!();
    println!("✅ Verification Status: {:?}", report.verification_status);
    println!("🎯 Failure Cause: {:?}", report.failure_cause);
    println!("📊 Process Score: {:.2} (not your fault!)", report.process_score.unwrap());
    println!("🎬 Outcome Success: {}", report.outcome_success.unwrap());
    println!();
    println!("💡 Recovery Hint: Environment blocked by CAPTCHA - cannot proceed.");
    println!("   Report to user or use alternative approach.");
    println!();
    println!();

    // Scenario 5: Out of Stock (Environment Block)
    println!("📋 Scenario 5: Out of Stock (Environment Block)");
    println!("{}", "-".repeat(80));
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::WindowTitleContains {
            substring: "Add to Cart".into(),
        },
        required: true,
    }];
    let evidence = json!({"title": "Product Currently Unavailable - Out of Stock"});
    let report = build_report(true, None, None, &expectations, &evidence);
    
    println!("Expected: Window title contains 'Add to Cart'");
    println!("Actual:   'Product Currently Unavailable - Out of Stock'");
    println!();
    println!("✅ Verification Status: {:?}", report.verification_status);
    println!("🎯 Failure Cause: {:?}", report.failure_cause);
    println!("📊 Process Score: {:.2} (product unavailable)", report.process_score.unwrap());
    println!("🎬 Outcome Success: {}", report.outcome_success.unwrap());
    println!();
    println!("💡 Recovery Hint: Resource unavailable - skip this item or notify user.");
    println!();
    println!();

    // Scenario 6: Success!
    println!("📋 Scenario 6: Success! (Everything Works)");
    println!("{}", "-".repeat(80));
    let expectations = vec![GuiExpectation {
        kind: GuiExpectationKind::UrlIs {
            url: "https://example.com/success".into(),
        },
        required: true,
    }];
    let evidence = json!({"url": "https://example.com/success"});
    let report = build_report(true, None, None, &expectations, &evidence);
    
    println!("Expected: https://example.com/success");
    println!("Actual:   https://example.com/success");
    println!();
    println!("✅ Verification Status: {:?}", report.verification_status);
    println!("🎯 Failure Cause: {:?}", report.failure_cause);
    println!("📊 Process Score: {:.2} (perfect execution!)", report.process_score.unwrap());
    println!("🎬 Outcome Success: {}", report.outcome_success.unwrap());
    println!();
    println!("💡 Recovery Hint: None needed - task completed successfully!");
    println!();
    println!();

    // Summary
    println!("{}", "=".repeat(80));
    println!("📊 Summary: Why Failure Attribution Matters");
    println!("{}", "=".repeat(80));
    println!();
    println!("✅ Agent Errors (process_score = 0.0):");
    println!("   - Wrong selector, bad value, incorrect action");
    println!("   - Agent should retry with corrected parameters");
    println!("   - This is a learning opportunity");
    println!();
    println!("🚫 Environment Blocks (process_score = 0.8):");
    println!("   - Login walls, CAPTCHAs, network errors, out-of-stock");
    println!("   - Agent did the RIGHT thing but was blocked");
    println!("   - Do NOT retry - adjust approach or report to user");
    println!();
    println!("❓ Evidence Absent (process_score = 0.8):");
    println!("   - Can't verify because data is missing");
    println!("   - Use pre_observe:auto or explicit evidence keys");
    println!();
    println!("🎯 Key Insight:");
    println!("   Process score separates 'how well did I execute?' from 'did it work?'");
    println!("   This prevents false penalties and enables better learning!");
    println!();
}
