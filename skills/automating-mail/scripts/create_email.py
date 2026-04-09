#!/usr/bin/env python3
"""
Create Email Script - PyXA Implementation
Creates and composes a new email message using Mail.app

Usage: python create_email.py "subject" "recipient@example.com" "body text"
"""

import sys
import PyXA

def create_email(subject, recipient, body):
    """Create and compose a new email message"""
    try:
        mail = PyXA.Application("Mail")

        # Create outgoing message
        message = mail.outgoing_messages().push({
            "subject": subject,
            "content": body
        })

        # Add recipient
        message.to_recipients = [recipient]

        # Make message visible for editing
        message.visible = True

        print(f"Email created with subject: {subject}")
        return True

    except Exception as e:
        print(f"Error creating email: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python create_email.py 'subject' 'recipient@example.com' 'body text'")
        sys.exit(1)

    subject = sys.argv[1]
    recipient = sys.argv[2]
    body = sys.argv[3]

    success = create_email(subject, recipient, body)
    sys.exit(0 if success else 1)