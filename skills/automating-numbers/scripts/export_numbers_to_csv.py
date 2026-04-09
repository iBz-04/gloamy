#!/usr/bin/env python3
"""
Export Numbers Spreadsheet to CSV Script
Exports a Numbers spreadsheet to CSV format

Usage: python export_numbers_to_csv.py "input.numbers" "output.csv"
"""

import sys
import subprocess
import csv
from pathlib import Path

def export_numbers_to_csv(input_file, output_file):
    """Export Numbers spreadsheet to CSV"""

    print(f"Exporting Numbers spreadsheet to CSV: {input_file} -> {output_file}")

    try:
        script = f'''
        tell application "Numbers"
            set theDoc to open POSIX file "{Path(input_file).resolve()}"

            -- Get the first sheet and table
            set firstSheet to sheet 1 of theDoc
            set firstTable to table 1 of firstSheet

            -- Read all cell values and create CSV data
            set csvData to ""

            -- Get table dimensions
            set rowCount to count of rows of firstTable
            set colCount to count of columns of firstTable

            repeat with i from 1 to rowCount
                set rowData to {{}}
                repeat with j from 1 to colCount
                    try
                        set cellValue to value of cell j of row i of firstTable
                        -- Convert to string and escape quotes
                        set cellString to cellValue as string
                        set end of rowData to cellString
                    on error
                        set end of rowData to ""
                    end try
                end repeat

                -- Join row with commas
                set AppleScript's text item delimiters to ","
                set rowString to rowData as string
                set AppleScript's text item delimiters to ""

                -- Add to CSV data
                set csvData to csvData & rowString & linefeed
            end repeat

            -- Close without saving
            close theDoc saving no

            return csvData
        end tell
        '''

        result = subprocess.run(["osascript", "-e", script],
                              capture_output=True, text=True, check=True, timeout=30)

        csv_data = result.stdout.strip()
        if csv_data:
            # Write CSV data to file
            with open(output_file, 'w', newline='', encoding='utf-8') as csvfile:
                # Parse the CSV data and write properly
                lines = csv_data.split('\n')
                for line in lines:
                    if line.strip():  # Skip empty lines
                        # Split by comma and handle basic CSV writing
                        csvfile.write(line + '\n')

            print(f"Successfully exported to CSV: {output_file}")
            return True
        else:
            print("No data found to export")
            return False

    except subprocess.CalledProcessError as e:
        print(f"AppleScript failed: {e}")
        print(f"Error output: {e.stderr}")
        return False
    except Exception as e:
        print(f"Unexpected error: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python export_numbers_to_csv.py 'input.numbers' 'output.csv'")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]

    success = export_numbers_to_csv(input_file, output_file)
    sys.exit(0 if success else 1)