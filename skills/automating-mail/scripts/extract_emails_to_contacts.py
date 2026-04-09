#!/usr/bin/env python3
"""
Extract Email Addresses to Contacts Script - PyXA Implementation
Extracts email addresses from selected Mail messages and adds them to Contacts

Usage: python extract_emails_to_contacts.py
"""

import PyXA
import re

def extract_email_addresses(text):
    """Extract email addresses from text using regex"""
    email_pattern = r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'
    return re.findall(email_pattern, text)

def add_to_contacts(email, name=None):
    """Add email address to Contacts app"""
    try:
        contacts = PyXA.Application("Contacts")

        # Check if contact already exists
        existing_contacts = contacts.contacts().filter(lambda c: email in str(c.email_addresses or []))

        if existing_contacts:
            print(f"Contact with email {email} already exists")
            return False

        # Create new contact
        contact_name = name or email.split('@')[0].replace('.', ' ').title()

        new_contact = contacts.contacts().push({
            "name": contact_name,
            "email_addresses": [email]
        })

        print(f"Added contact: {contact_name} ({email})")
        return True

    except Exception as e:
        print(f"Error adding contact: {e}")
        return False

def main():
    """Main function to extract emails from selected messages"""
    try:
        mail = PyXA.Application("Mail")

        # Get selected messages
        selected_messages = mail.selection()

        if not selected_messages:
            print("No messages selected. Please select one or more messages in Mail.app")
            return False

        total_added = 0

        for message in selected_messages:
            # Extract emails from various fields
            all_text = f"{message.sender or ''} {message.subject or ''} {message.content or ''}"

            emails = extract_email_addresses(all_text)

            for email in emails:
                # Skip the sender's own email
                if message.sender and email in message.sender:
                    continue

                if add_to_contacts(email):
                    total_added += 1

        print(f"Successfully added {total_added} contacts from {len(selected_messages)} messages")
        return True

    except Exception as e:
        print(f"Error processing messages: {e}")
        return False

if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)