#!/usr/bin/env python3
"""
Export Excel to CSV Script - PyXA Implementation
Exports Excel worksheets to CSV files

Usage: python export_excel_to_csv.py "workbook.xlsx" [output_directory]
"""

import sys
import os
import subprocess
from pathlib import Path

def export_excel_to_csv(excel_file, output_dir=None):
    """Export Excel workbook to CSV files using AppleScript"""
    try:
        abs_input = str(Path(excel_file).resolve())
        
        # Set output directory
        if not output_dir:
            output_dir = os.path.dirname(abs_input) or "."
        
        abs_output_dir = str(Path(output_dir).resolve())
        os.makedirs(abs_output_dir, exist_ok=True)

        # In integration tests, output_dir might be a filename or a directory
        if output_dir.endswith('.csv'):
            abs_output_csv = str(Path(output_dir).resolve())
        else:
            abs_output_csv = os.path.join(abs_output_dir, "ExportedData.csv")

        script = f'''
        tell application "Microsoft Excel"
            set wb to open (POSIX file "{abs_input}")
            save as wb filename (POSIX file "{abs_output_csv}") file format CSV file format
            close wb saving no
        end tell
        '''
        
        subprocess.run(["osascript", "-e", script], check=True, timeout=30)
        print(f"Successfully exported to CSV: {abs_output_csv}")
        return True

    except Exception as e:
        print(f"Export failed: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python export_excel_to_csv.py 'workbook.xlsx' [output_directory]")
        sys.exit(1)

    excel_file = sys.argv[1]
    output_dir = sys.argv[2] if len(sys.argv) > 2 else None

    success = export_excel_to_csv(excel_file, output_dir)
    sys.exit(0 if success else 1)
