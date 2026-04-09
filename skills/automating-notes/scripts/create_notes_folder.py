#!/usr/bin/env python3
"""
Create Notes Folder Script - PyXA Implementation
Creates a new folder in Apple Notes

Usage: python create_notes_folder.py "Folder Name" ["Account Name"]
"""

import sys
import PyXA

def create_notes_folder(folder_name, account_name=None):
    """Create a new folder in Notes app"""
    try:
        notes = PyXA.Application("Notes")

        # Find target account
        target_account = None

        if account_name:
            # Find specific account
            for account in notes.accounts():
                if account.name == account_name:
                    target_account = account
                    break
        else:
            # Use first account (usually iCloud)
            target_account = notes.accounts()[0]

        if not target_account:
            print(f"Account '{account_name}' not found" if account_name else "No accounts found")
            return False

        # Check if folder already exists
        existing_folders = target_account.folders()
        for folder in existing_folders:
            if folder.name == folder_name:
                print(f"Folder '{folder_name}' already exists in account '{target_account.name}'")
                return True

        # Create new folder
        new_folder = target_account.folders().push({
            "name": folder_name
        })

        print(f"Created folder '{folder_name}' in account '{target_account.name}'")
        return True

    except Exception as e:
        print(f"Error creating notes folder: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python create_notes_folder.py 'Folder Name' ['Account Name']")
        sys.exit(1)

    folder_name = sys.argv[1]
    account_name = sys.argv[2] if len(sys.argv) > 2 else None

    success = create_notes_folder(folder_name, account_name)
    sys.exit(0 if success else 1)