#!/usr/bin/env python3
"""
Create Excel Spreadsheet Script - PyXA Implementation
Creates a new Excel spreadsheet with sample data

Usage: python create_excel_spreadsheet.py "Workbook Name" [sheet_count] [save_path]
"""

import sys
import PyXA
import os
import subprocess

def create_excel_spreadsheet(workbook_name, sheet_count=1, save_path=None):
    """Create a new Excel spreadsheet with sample data"""
    try:
        # Use AppleScript directly for reliability in this script
        # as PyXA Excel support can be inconsistent
        raise Exception("Force AppleScript fallback for reliability")

        excel = PyXA.Application("Microsoft Excel")

        # Create new workbook
        try:
            workbook = excel.make("workbook")
        except Exception:
            workbook = excel.workbooks().push(excel.class_("workbook"))

        # Name workbook if possible
        try:
            workbook.name = workbook_name
        except Exception:
            pass

        # Add sheets if requested
        try:
            while len(workbook.worksheets()) < sheet_count:
                workbook.worksheets().push()
        except Exception as e:
            print(f"Warning: could not add sheets: {e}")

        # Add sample data to first sheet (best effort)
        try:
            sheet = workbook.worksheets()[0]
            sample_data = [
                ["Product", "Price", "Quantity", "Total"],
                ["Widget A", 10.99, 5, "=B2*C2"],
                ["Widget B", 15.50, 3, "=B3*C3"],
                ["Widget C", 8.75, 10, "=B4*C4"],
                ["", "", "", "=SUM(D2:D4)"]
            ]
            for row_idx, row_data in enumerate(sample_data, start=1):
                for col_idx, cell_value in enumerate(row_data, start=1):
                    try:
                        sheet.cells(row_idx, col_idx).value = cell_value
                    except Exception:
                        pass
        except Exception as e:
            print(f"Warning: could not write sample data: {e}")

        # Save workbook
        if save_path:
            save_path = os.path.abspath(save_path)
            workbook.save({"in": save_path})
            print(f"Saved to: {save_path}")
        else:
            workbook.save()

        print(f"Created Excel workbook '{workbook_name}' with {sheet_count} sheet(s)")
        return True

    except Exception as e:
        print(f"PyXA path failed: {e}; attempting AppleScript fallback.")
        if not save_path:
            print("No save path provided; cannot fallback to AppleScript save.")
            return False
        save_path = os.path.abspath(save_path)
        try:
            script = f'''
            tell application "Microsoft Excel"
                set wb to make new workbook
                tell worksheet 1 of wb
                    set value of cell 1 of row 1 to "Product"
                    set value of cell 2 of row 1 to "Price"
                end tell
                save wb in (POSIX file "{save_path}")
                close wb saving no
            end tell
            '''
            subprocess.run(["osascript", "-e", script], check=True, timeout=30)
            print(f"Saved to: {save_path} (AppleScript fallback)")
            return True
        except Exception as e2:
            print(f"AppleScript fallback failed: {e2}")
            return False

if __name__ == "__main__":
    workbook_name = sys.argv[1] if len(sys.argv) > 1 else "Integration Test Workbook"
    sheet_count = int(sys.argv[2]) if len(sys.argv) > 2 else 1
    save_path = sys.argv[3] if len(sys.argv) > 3 else None

    success = create_excel_spreadsheet(workbook_name, sheet_count, save_path)
    sys.exit(0 if success else 1)
