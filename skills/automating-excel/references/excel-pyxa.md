# Excel Automation with PyXA (Python)

PyXA provides a Pythonic interface to control Microsoft Excel on macOS via Apple's Scripting Bridge.

## Installation

```bash
pip install pyxa
```

## Bootstrapping

```python
import PyXA

# Launch or activate Excel
excel = PyXA.Application("Microsoft Excel")
excel.activate()
```

## Workbook Management

### Creating Workbooks

```python
# Create new workbook
wb = excel.workbooks.new()

# Access first sheet
sheet = wb.sheets[0]
```

### Opening Workbooks

```python
# Open existing file
wb = excel.open("/Users/me/Documents/Data.xlsx")
```

### Saving Workbooks

```python
# Save to new location
wb.save_as("/Users/me/Desktop/Report.xlsx")

# Save existing
wb.save()

# Close without saving
wb.close(save=False)
```

## Cell Operations

### Reading and Writing Values

```python
# Single cell
sheet.cells.item("A1").value = "Header"
value = sheet.cells.item("A1").value

# Using row/column indices (1-based)
sheet.cells.item(1, 1).value = "Row 1, Col 1"
```

### Working with Ranges

```python
# Range of cells
sheet.range("A1:C10").value = data_matrix

# Read range values (returns 2D list)
values = sheet.range("A1:C10").value
```

### Formulas

```python
# Set formula
sheet.cells.item("D1").formula = "=SUM(A1:C1)"
sheet.cells.item("E1").formula = "=B1*2"
```

## Batch Data Operations

```python
# Write 2D array to range (efficient)
data = [
    ["Name", "Score", "Grade"],
    ["Alice", 95, "A"],
    ["Bob", 87, "B"],
    ["Carol", 92, "A"]
]

# Calculate range size
rows = len(data)
cols = len(data[0])

# Write in one operation
sheet.range(f"A1:C{rows}").value = data
```

## Formatting

### Cell Formatting

```python
# Bold headers
sheet.range("A1:C1").font_bold = True

# Background color
sheet.range("A1:C1").interior_color = PyXA.XAColor.blue

# Font properties
sheet.range("A1:C1").font_size = 14
sheet.range("A1:C1").font_name = "Arial"
```

### Column Width

```python
# Set specific width
sheet.columns.item("A").width = 20

# Auto-fit columns
sheet.columns.item("A:C").autofit()
```

### Borders

```python
# Add borders to range
sheet.range("A1:C10").borders.line_style = "continuous"
sheet.range("A1:C10").borders.weight = "thin"
```

## Export to PDF

```python
# Export sheet to PDF
sheet.export("/Users/me/Desktop/Report.pdf",
             export_format=PyXA.XL_ExportFormat.pdf)
```

## Complete Report Generator

```python
import PyXA

def generate_excel_report(output_path, headers, data):
    """Generate a formatted Excel report with PyXA."""
    excel = PyXA.Application("Microsoft Excel")
    excel.activate()

    # Create workbook
    wb = excel.workbooks.new()
    sheet = wb.sheets[0]
    sheet.name = "Report"

    # Prepare data matrix
    rows = [headers] + data
    row_count = len(rows)
    col_count = len(headers)

    # Write data in single operation
    sheet.range(f"A1:{chr(64+col_count)}{row_count}").value = rows

    # Format headers
    header_range = f"A1:{chr(64+col_count)}1"
    sheet.range(header_range).font_bold = True
    sheet.range(header_range).interior_color = PyXA.XAColor.blue
    sheet.range(header_range).font_color = PyXA.XAColor.white

    # Auto-fit columns
    for i in range(1, col_count + 1):
        sheet.columns.item(i).autofit()

    # Add borders to data area
    data_range = f"A1:{chr(64+col_count)}{row_count}"
    sheet.range(data_range).borders.line_style = "continuous"

    # Save
    wb.save_as(output_path)

    return {"success": True, "path": output_path}

# Usage
headers = ["ID", "Name", "Value", "Status"]
data = [
    [1, "Alpha", 100, "Active"],
    [2, "Beta", 200, "Pending"],
    [3, "Gamma", 150, "Active"]
]
generate_excel_report("/Users/me/Desktop/Report.xlsx", headers, data)
```

## CSV Import Pattern

```python
import PyXA
import csv

def csv_to_excel(csv_path, output_path):
    """Convert CSV file to formatted Excel workbook."""
    # Read CSV
    with open(csv_path, 'r') as f:
        reader = csv.reader(f)
        data = list(reader)

    # Create Excel workbook
    excel = PyXA.Application("Microsoft Excel")
    excel.activate()

    wb = excel.workbooks.new()
    sheet = wb.sheets[0]

    # Calculate dimensions
    rows = len(data)
    cols = len(data[0]) if data else 0

    # Write data
    if rows > 0 and cols > 0:
        sheet.range(f"A1:{chr(64+cols)}{rows}").value = data

    # Format header row
    sheet.range(f"A1:{chr(64+cols)}1").font_bold = True

    # Save
    wb.save_as(output_path)
    wb.close()

# Usage
csv_to_excel("/Users/me/data.csv", "/Users/me/data.xlsx")
```

## PyXA vs JXA Comparison

| Operation | PyXA | JXA |
|-----------|------|-----|
| Set cell | `cell.value = x` | `cell.value = x` |
| Get cell | `x = cell.value` | `x = cell.value()` |
| Range write | `range.value = [[...]]` | `range.value = [[...]]` |
| Formatting | `range.font_bold = True` | `range.fontObject.bold = true` |

## Notes

- PyXA and JXA both use Scripting Bridge (similar performance)
- Excel ranges are 1-indexed in both APIs
- 2D arrays required for multi-cell operations
- Always match array dimensions to range dimensions
- Use `wb.close(save=False)` to discard changes
