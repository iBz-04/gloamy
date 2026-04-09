#!/usr/bin/env python3
"""
Search Notes Script - PyXA Implementation
Searches for notes containing specific text

Usage: python search_notes.py "search term" ["folder name"]
"""

import sys
import PyXA

def search_notes(search_term, folder_name=None):
    """Search for notes containing the search term"""
    try:
        notes = PyXA.Application("Notes")

        matching_notes = []

        for account in notes.accounts():
            folders_to_search = []

            if folder_name:
                # Search specific folder
                for folder in account.folders():
                    if folder.name == folder_name:
                        folders_to_search = [folder]
                        break
            else:
                # Search all folders
                folders_to_search = account.folders()

            for folder in folders_to_search:
                try:
                    folder_notes = folder.notes()

                    for note in folder_notes:
                        # Check title and body
                        title = note.name or ""
                        body = note.body or ""

                        if (search_term.lower() in title.lower() or
                            search_term.lower() in body.lower()):
                            matching_notes.append({
                                'title': title,
                                'folder': folder.name,
                                'account': account.name,
                                'id': note.id
                            })

                except Exception as e:
                    print(f"Error searching folder {folder.name}: {e}")
                    continue

        # Display results
        if matching_notes:
            print(f"Found {len(matching_notes)} notes containing '{search_term}':")
            for i, note in enumerate(matching_notes, 1):
                print(f"{i}. '{note['title']}' in {note['account']} > {note['folder']}")
        else:
            print(f"No notes found containing '{search_term}'")

        return matching_notes

    except Exception as e:
        print(f"Error searching notes: {e}")
        return []

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python search_notes.py 'search term' ['folder name']")
        sys.exit(1)

    search_term = sys.argv[1]
    folder = sys.argv[2] if len(sys.argv) > 2 else None

    results = search_notes(search_term, folder)
    sys.exit(0 if results else 1)