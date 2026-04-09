#!/usr/bin/env python3
"""
Create Numbers Spreadsheet Script - PyXA Implementation
Creates a new Numbers spreadsheet with sample data

Usage: python create_numbers_spreadsheet.py "Spreadsheet Name" ["Save Path"]
"""

import sys
import subprocess
from pathlib import Path

def create_numbers_spreadsheet(name, save_path=None):
    """Create a new Numbers spreadsheet with sample data"""

    print("Creating Numbers spreadsheet with AppleScript...")

    # Use AppleScript for Numbers automation since PyXA Numbers support may be limited
    try:
        # Build the script based on whether we have a save path
        if save_path:
            abs_path = str(Path(save_path).resolve())
            save_command = f'save newDoc in POSIX file "{abs_path}"'
        else:
            save_command = '-- no save'

        script = f'''
        tell application "Numbers"
            activate
            -- Create new document
            set newDoc to make new document

            -- Get the first sheet and table
            set firstSheet to sheet 1 of newDoc
            set firstTable to table 1 of firstSheet

            -- Set simple headers
            set value of cell 1 of row 1 of firstTable to "Product"
            set value of cell 2 of row 1 of firstTable to "Price"

            -- Add simple data
            set value of cell 1 of row 2 of firstTable to "Widget A"
            set value of cell 2 of row 2 of firstTable to 10.99

            -- Save if requested, otherwise close
            if "{save_command}" is not "-- no save" then
                {save_command}
                close newDoc saving no
            else
                close newDoc saving no
            end if
        end tell
        '''

        subprocess.run(["osascript", "-e", script], check=True, timeout=30)
        print(f"Successfully created Numbers spreadsheet: {name}")
        if save_path:
            print(f"Saved to: {save_path}")
        return True
        print(f"Successfully created Numbers spreadsheet: {name}")
        if save_path:
            print(f"Saved to: {save_path}")
        return True

    except subprocess.CalledProcessError as e:
        print(f"AppleScript failed: {e}")
        return False
    except Exception as e:
        print(f"Unexpected error: {e}")
        return False

if __name__ == "__main__":
    name = sys.argv[1] if len(sys.argv) > 1 else "Sample Spreadsheet"
    save_path = sys.argv[2] if len(sys.argv) > 2 else None

    success = create_numbers_spreadsheet(name, save_path)
    sys.exit(0 if success else 1)