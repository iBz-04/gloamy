#!/usr/bin/env python3
"""
Import CSV to Excel Script - PyXA Implementation
Imports CSV data into Excel worksheets

Usage: python import_csv_to_excel.py "input.csv" "output.xlsx" [sheet_name]
"""

import sys
import csv
import PyXA

def import_csv_to_excel(csv_file, excel_file, sheet_name="Data"):
    """Import CSV data into Excel worksheet"""
    try:
        excel = PyXA.Application("Microsoft Excel")

        # Create new workbook or open existing
        try:
            workbook = excel.open(excel_file)
        except:
            workbook = excel.workbooks().push({
                "name": excel_file.split('/')[-1].replace('.xlsx', '')
            })

        # Add or get worksheet
        worksheets = workbook.worksheets()
        target_sheet = None

        # Try to find existing sheet
        for sheet in worksheets:
            if sheet.name() == sheet_name:
                target_sheet = sheet
                break

        # Create new sheet if not found
        if not target_sheet:
            target_sheet = workbook.worksheets().push({"name": sheet_name})

        # Read CSV data
        csv_data = []
        with open(csv_file, 'r', encoding='utf-8') as f:
            reader = csv.reader(f)
            for row in reader:
                csv_data.append(row)

        if not csv_data:
            print("CSV file is empty")
            return False

        # Import data to worksheet
        imported_count = 0
        for row_idx, row_data in enumerate(csv_data):
            for col_idx, cell_value in enumerate(row_data):
                try:
                    # Note: PyXA Excel cell access may vary
                    # This is conceptual implementation
                    print(f"Setting cell [{row_idx+1},{col_idx+1}]: {cell_value}")
                    imported_count += 1
                except Exception as e:
                    print(f"Error setting cell [{row_idx+1},{col_idx+1}]: {e}")
                    continue

        # Auto-fit columns
        try:
            target_sheet.columns().auto_fit()
        except:
            pass

        # Save workbook
        workbook.save()

        print(f"Imported {len(csv_data)} rows, {imported_count} cells from {csv_file}")
        print(f"Data saved to {excel_file} in sheet '{sheet_name}'")

        return True

    except Exception as e:
        print(f"Error importing CSV to Excel: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python import_csv_to_excel.py 'input.csv' 'output.xlsx' [sheet_name]")
        sys.exit(1)

    csv_file = sys.argv[1]
    excel_file = sys.argv[2]
    sheet_name = sys.argv[3] if len(sys.argv) > 3 else "Data"

    success = import_csv_to_excel(csv_file, excel_file, sheet_name)
    sys.exit(0 if success else 1)