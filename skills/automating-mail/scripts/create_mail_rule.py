#!/usr/bin/env python3
"""
Create Mail Rule Script - PyXA Implementation
Creates a mail rule to automatically organize incoming emails

Usage: python create_mail_rule.py "Rule Name" "sender@domain.com" "target mailbox"
"""

import sys
import PyXA

def create_mail_rule(rule_name, sender_condition, target_mailbox):
    """Create a mail rule for automatic organization"""
    try:
        mail = PyXA.Application("Mail")

        # Note: Mail rules are not directly scriptable via PyXA
        # This script provides a template for manual rule creation
        # and demonstrates how to set up the conditions

        print(f"Creating mail rule: {rule_name}")
        print(f"Condition: From contains '{sender_condition}'")
        print(f"Action: Move to mailbox '{target_mailbox}'")
        print("\nTo create this rule manually in Mail.app:")
        print("1. Mail > Preferences > Rules")
        print("2. Click '+' to add a new rule")
        print(f"3. Description: {rule_name}")
        print("4. If ANY of the following conditions are met:")
        print(f"   • From contains: {sender_condition}")
        print("5. Perform the following actions:")
        print(f"   • Move Message to mailbox: {target_mailbox}")
        print("6. Click OK to save")

        # In a real implementation, you might use AppleScript
        # or UI scripting to automate rule creation
        applescript_rule = f'''
        tell application "Mail"
            make new rule with properties {{
                name: "{rule_name}",
                sender contains: "{sender_condition}",
                move message: mailbox "{target_mailbox}" of account 1
            }}
        end tell
        '''

        print("\nEquivalent AppleScript:")
        print(applescript_rule)

        return True

    except Exception as e:
        print(f"Error creating mail rule template: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python create_mail_rule.py 'Rule Name' 'sender@domain.com' 'target mailbox'")
        sys.exit(1)

    rule_name = sys.argv[1]
    sender = sys.argv[2]
    mailbox = sys.argv[3]

    success = create_mail_rule(rule_name, sender, mailbox)
    sys.exit(0 if success else 1)