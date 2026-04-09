#!/usr/bin/env python3
"""
Read Numbers Spreadsheet Script
Reads data from a Numbers spreadsheet

Usage: python read_numbers_spreadsheet.py "path/to/spreadsheet.numbers"
"""

import sys
import subprocess
import json
from pathlib import Path

def read_numbers_spreadsheet(file_path):
    """Read data from a Numbers spreadsheet"""

    print(f"Reading Numbers spreadsheet: {file_path}")

    try:
        script = f'''
        tell application "Numbers"
            set theDoc to open POSIX file "{Path(file_path).resolve()}"

            -- Get the first sheet and table
            set firstSheet to sheet 1 of theDoc
            set firstTable to table 1 of firstSheet

            -- Read all cell values
            set allData to {{}}

            -- Get table dimensions (approximate)
            set rowCount to count of rows of firstTable
            set colCount to count of columns of firstTable

            repeat with i from 1 to rowCount
                set rowData to {{}}
                repeat with j from 1 to colCount
                    try
                        set cellValue to value of cell j of row i of firstTable
                        set end of rowData to cellValue
                    on error
                        set end of rowData to ""
                    end try
                end repeat
                set end of allData to rowData
            end repeat

            -- Close without saving
            close theDoc saving no

            return allData
        end tell
        '''

        result = subprocess.run(["osascript", "-e", script],
                              capture_output=True, text=True, check=True, timeout=30)

        # Parse the AppleScript result
        # AppleScript returns data in a format that needs parsing
        output = result.stdout.strip()
        if output:
            print(f"Raw output: {output}")
            # For now, just indicate success
            print("Successfully read Numbers spreadsheet data")
            return True
        else:
            print("No data found in spreadsheet")
            return False

    except subprocess.CalledProcessError as e:
        print(f"AppleScript failed: {e}")
        print(f"Error output: {e.stderr}")
        return False
    except Exception as e:
        print(f"Unexpected error: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python read_numbers_spreadsheet.py 'path/to/spreadsheet.numbers'")
        sys.exit(1)

    file_path = sys.argv[1]
    success = read_numbers_spreadsheet(file_path)
    sys.exit(0 if success else 1)