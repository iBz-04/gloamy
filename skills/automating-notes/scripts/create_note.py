#!/usr/bin/env python3
"""
Create Note Script - PyXA Implementation
Creates a new note in Apple Notes

Usage: python create_note.py "Note Title" "Note Content" ["Folder Name"]
"""

import sys
import PyXA

def create_note(title, content, folder_name=None):
    """Create a new note in Notes app"""
    try:
        notes = PyXA.Application("Notes")

        # Find or create target folder
        target_folder = None

        if folder_name:
            # Try to find existing folder
            for account in notes.accounts():
                for folder in account.folders():
                    if folder.name == folder_name:
                        target_folder = folder
                        break
                if target_folder:
                    break

            # Create folder if it doesn't exist
            if not target_folder:
                # Create in first account
                first_account = notes.accounts()[0]
                target_folder = first_account.folders().push({"name": folder_name})

        if not target_folder:
            # Use default folder (first account's default)
            first_account = notes.accounts()[0]
            target_folder = first_account.folders()[0]  # Default folder

        # Create the note
        note = target_folder.notes().push({
            "name": title,
            "body": content
        })

        # Make it visible
        note.show()

        print(f"Created note '{title}' in folder '{folder_name or 'default'}'")
        return True

    except Exception as e:
        print(f"Error creating note: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python create_note.py 'Note Title' 'Note Content' ['Folder Name']")
        sys.exit(1)

    title = sys.argv[1]
    content = sys.argv[2]
    folder = sys.argv[3] if len(sys.argv) > 3 else None

    success = create_note(title, content, folder)
    sys.exit(0 if success else 1)