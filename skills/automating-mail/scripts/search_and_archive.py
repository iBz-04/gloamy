#!/usr/bin/env python3
"""
Mail Search and Archive Script - PyXA Implementation
Searches for emails matching criteria and moves them to archive

Usage: python search_and_archive.py "search term" "archive mailbox name"
"""

import sys
import PyXA

def search_and_archive(search_term, archive_mailbox="Archive"):
    """Search for emails and move them to archive"""
    try:
        mail = PyXA.Application("Mail")

        # Find all accounts
        accounts = mail.accounts()

        archived_count = 0

        for account in accounts:
            print(f"Searching in account: {account.name}")

            # Get all mailboxes for this account
            mailboxes = account.mailboxes()

            # Find archive mailbox
            archive_box = None
            for mailbox in mailboxes:
                if archive_mailbox.lower() in mailbox.name.lower():
                    archive_box = mailbox
                    break

            if not archive_box:
                print(f"Archive mailbox '{archive_mailbox}' not found in account {account.name}")
                continue

            # Search through all mailboxes for messages
            for mailbox in mailboxes:
                try:
                    messages = mailbox.messages()

                    # Filter messages containing search term
                    matching_messages = []
                    for msg in messages:
                        if (search_term.lower() in (msg.subject or "").lower() or
                            search_term.lower() in (msg.content or "").lower()):
                            matching_messages.append(msg)

                    # Move matching messages to archive
                    for msg in matching_messages:
                        msg.move_to(archive_box)
                        archived_count += 1
                        print(f"Archived: {msg.subject}")

                except Exception as e:
                    print(f"Error processing mailbox {mailbox.name}: {e}")
                    continue

        print(f"Total messages archived: {archived_count}")
        return True

    except Exception as e:
        print(f"Error in search and archive: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python search_and_archive.py 'search term' ['archive mailbox name']")
        sys.exit(1)

    search_term = sys.argv[1]
    archive_name = sys.argv[2] if len(sys.argv) > 2 else "Archive"

    success = search_and_archive(search_term, archive_name)
    sys.exit(0 if success else 1)