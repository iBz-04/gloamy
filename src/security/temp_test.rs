#[cfg(test)]
mod temp {
    use crate::security::policy::{SecurityPolicy, AutonomyLevel};
    #[test]
    fn test_user_script() {
        let p = SecurityPolicy {
            allowed_commands: vec!["osascript".into()],
            ..SecurityPolicy::default()
        };
        let cmd = r#"osascript <<'APPLESCRIPT'
tell application "Reminders"
    activate
    if (count of lists) is 0 then error "No reminder lists found"
    set targetList to first list
    
    set newReminder to make new reminder with properties {name:"outing", due date:(current date) + 3600}
    tell targetList
        move newReminder to end of reminders
    end tell
end tell
APPLESCRIPT"#;
        
        // Print why it failed by running through the checks manually
        println!("Contains backtick: {}", cmd.contains('`'));
        // contains_unquoted_shell_variable_expansion is private, wait I can just call it by name if it's in the same module... Wait, I'm in a sub-module.
    }
}
