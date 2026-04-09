# PyXA Numbers Module API Reference

> **New in PyXA version 0.0.8** - Control macOS Numbers using JXA-like syntax from Python.

This reference documents all classes, methods, properties, and enums in the PyXA Numbers module. Numbers inherits table, cell, row, column, and range functionality from the shared iWork base classes. For practical examples and usage patterns, see [numbers-basics.md](numbers-basics.md).

## Contents

- [Class Hierarchy](#class-hierarchy)
- [XANumbersApplication](#xanumbersapplication)
- [XANumbersDocument](#xanumbersdocument)
- [XANumbersSheet](#xanumberssheet)
- [XANumbersTemplate](#xanumberstemplate)
- [XANumbersWindow](#xanumberswindow)
- [XANumbersContainer](#xanumberscontainer)
- [Table Classes (iWork Base)](#table-classes-iwork-base)
  - [XAiWorkTable](#xaiworktable)
  - [XAiWorkRange](#xaiworkrange)
  - [XAiWorkRow](#xaiworkrow)
  - [XAiWorkColumn](#xaiworkcolumn)
  - [XAiWorkCell](#xaiworkcell)
  - [XAiWorkChart](#xaiworkchart)
- [List Classes](#list-classes)
- [Enumerations](#enumerations)
- [Quick Reference Tables](#quick-reference-tables)

---

## Class Hierarchy

```
XAObject
├── XANumbersApplication (XAiWorkApplication)
│   ├── XANumbersDocument (XAiWorkDocument)
│   │   ├── XANumbersSheet (XANumbersContainer)
│   │   │   ├── XAiWorkTable (XAiWorkiWorkItem)
│   │   │   │   ├── XAiWorkRange
│   │   │   │   │   ├── XAiWorkRow
│   │   │   │   │   ├── XAiWorkColumn
│   │   │   │   │   └── XAiWorkCell
│   │   │   │   └── XAiWorkChart
│   │   │   └── [images, shapes, lines, etc.]
│   │   └── XANumbersTemplate
│   └── XANumbersWindow (XAiWorkWindow)
└── XANumbersContainerList
```

---

## XANumbersApplication

**Bases:** `XAiWorkApplication`

Main entry point for interacting with Numbers.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Application name ("Numbers") |
| `frontmost` | `bool` | Whether Numbers is frontmost application |
| `version` | `str` | Application version |
| `current_document` | `XANumbersDocument` | Currently active document |

### Methods

#### `documents(filter=None) -> XANumbersDocumentList`

Returns a list of open documents matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
import PyXA
numbers = PyXA.Application("Numbers")
docs = numbers.documents()
for doc in docs:
    print(doc.name)
```

#### `templates(filter=None) -> XANumbersTemplateList`

Returns available templates matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
templates = numbers.templates()
print(templates.name())
# ['Blank', 'Checklist', 'Invoice', ...]
```

#### `new_document(file_path='./Untitled.numbers', template=None) -> XANumbersDocument`

Creates a new document.

**Parameters:**
- `file_path` (`str | XAPath`) - Path to create document at
- `template` (`XANumbersTemplate | None`) - Template to initialize with

**Returns:** The newly created document

**Example:**
```python
# Create blank document
doc = numbers.new_document("~/Documents/report.numbers")

# Create from template
template = numbers.templates().by_name("Invoice")
doc = numbers.new_document("~/Documents/invoice.numbers", template=template)
```

#### `new_sheet(document, properties=None) -> XANumbersSheet`

Creates a new sheet in the specified document.

**Parameters:**
- `document` (`XANumbersDocument`) - Document to add sheet to
- `properties` (`dict | None`) - Properties for the new sheet

**Returns:** The newly created sheet

#### `make(specifier, properties=None, data=None)`

Creates a new element without adding to any list. Use `XAList.push()` to add.

**Parameters:**
- `specifier` (`str | ObjectType`) - Class name to create
- `properties` (`dict`) - Properties for the object
- `data` (`Any`) - Initialization data

**Example:**
```python
# Create a new table
new_table = numbers.make("table", {"name": "Sales Data", "row_count": 10, "column_count": 5})
doc.sheets()[0].tables().push(new_table)

# Create a new line
new_line = numbers.make("line", {"startPoint": (100, 100), "endPoint": (200, 200)})
doc.sheets()[0].lines().push(new_line)
```

---

## XANumbersDocument

**Bases:** `XAiWorkDocument`

Represents an open Numbers spreadsheet.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Document name |
| `id` | `str` | Unique identifier |
| `file` | `XAPath` | Location on disk (if saved) |
| `modified` | `bool` | Whether modified since last save |
| `password_protected` | `bool` | Whether password protected |
| `document_template` | `XANumbersTemplate` | Assigned template |
| `active_sheet` | `XANumbersSheet` | Currently active sheet (read/write) |
| `selection` | `XAiWorkiWorkItemList` | Currently selected items |
| `properties` | `dict` | All document properties |

### Methods

#### `sheets(filter=None) -> XANumbersSheetList`

Returns sheets matching the filter.

**Example:**
```python
doc = numbers.documents()[0]
all_sheets = doc.sheets()
first_sheet = doc.sheets()[0]
named_sheet = doc.sheets().by_name("Sales")
```

#### `new_sheet(properties=None) -> XANumbersSheet`

Creates a new sheet at the end of the document.

**Parameters:**
- `properties` (`dict`) - Properties for the new sheet (e.g., `{"name": "Q4 Data"}`)

**Returns:** The newly created sheet

**Example:**
```python
new_sheet = doc.new_sheet({"name": "Summary"})
```

#### `export(file_path=None, format=ExportFormat.PDF)`

Exports the spreadsheet.

**Parameters:**
- `file_path` (`str | XAPath | None`) - Export destination
- `format` (`ExportFormat`) - Export format (default: PDF)

**Example:**
```python
# Export to PDF
doc.export("/path/to/output.pdf", XANumbersApplication.ExportFormat.PDF)

# Export to Excel
doc.export("/path/to/output.xlsx", XANumbersApplication.ExportFormat.MICROSOFT_EXCEL)

# Export to CSV
doc.export("/path/to/output.csv", XANumbersApplication.ExportFormat.CSV)
```

#### `save()`

Saves the document in Numbers format.

**Example:**
```python
doc.save()
```

---

## XANumbersSheet

**Bases:** `XANumbersContainer`

Represents a single sheet in a Numbers spreadsheet.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Sheet name (read/write) |
| `properties` | `dict` | All sheet properties |

### Methods

#### `tables(filter=None) -> XAiWorkTableList`

Returns tables in the sheet.

**Example:**
```python
sheet = doc.sheets()[0]
tables = sheet.tables()
first_table = tables[0]
named_table = tables.by_name("Data")
```

#### `charts(filter=None) -> XAiWorkChartList`

Returns charts in the sheet.

#### `images(filter=None) -> XAiWorkImageList`

Returns images in the sheet.

#### `shapes(filter=None) -> XAiWorkShapeList`

Returns shapes in the sheet.

#### `lines(filter=None) -> XAiWorkLineList`

Returns lines in the sheet.

#### `text_items(filter=None) -> XAiWorkTextItemList`

Returns text items in the sheet.

#### `groups(filter=None) -> XAiWorkGroupList`

Returns groups in the sheet.

#### `iwork_items(filter=None) -> XAiWorkiWorkItemList`

Returns all iWork items in the sheet.

#### `audio_clips(filter=None) -> XAiWorkAudioClipList`

Returns audio clips in the sheet.

#### `movies(filter=None) -> XAiWorkMovieList`

Returns movies in the sheet.

#### `add_image(file_path) -> XAiWorkImage`

Adds an image to the sheet.

**Parameters:**
- `file_path` (`str | XAPath | XAImage`) - Path to image file

**Returns:** The newly created image object

**Example:**
```python
image = sheet.add_image("/path/to/chart.png")
```

---

## XANumbersTemplate

**Bases:** `XAObject`

Represents a Numbers template.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `name` | `str` | Template name |

---

## XANumbersWindow

**Bases:** `XAiWorkWindow`

Represents a Numbers window.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `document` | `XANumbersDocument` | Document displayed in window |

---

## XANumbersContainer

**Bases:** `XAiWorkContainer`

Base class for containers (sheets) in Numbers.

---

## Table Classes (iWork Base)

Numbers uses shared iWork base classes for tables, cells, rows, columns, and ranges. These classes are defined in `iWorkApplicationBase` and inherited by Numbers.

### XAiWorkTable

**Bases:** `XAiWorkiWorkItem`

Represents a table in a Numbers sheet.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Table identifier (read/write) |
| `row_count` | `int` | Number of rows (read/write) |
| `column_count` | `int` | Number of columns (read/write) |
| `header_row_count` | `int` | Number of header rows (read/write) |
| `header_column_count` | `int` | Number of header columns (read/write) |
| `footer_row_count` | `int` | Number of footer rows (read/write) |
| `cell_range` | `XAiWorkRange` | All cells in the table |
| `selection_range` | `XAiWorkRange` | Currently selected cells (read/write) |

#### Methods

##### `cells(filter=None) -> XAiWorkCellList`

Returns all cells in the table.

**Example:**
```python
table = sheet.tables()[0]
all_cells = table.cells()
cell_a1 = table.cells().by_name("A1")
```

##### `rows(filter=None) -> XAiWorkRowList`

Returns all rows in the table.

**Example:**
```python
rows = table.rows()
first_row = rows[0]
row_by_address = rows.by_address(5)  # Get row 5
```

##### `columns(filter=None) -> XAiWorkColumnList`

Returns all columns in the table.

**Example:**
```python
columns = table.columns()
first_column = columns[0]
col_by_address = columns.by_address(2)  # Get column B (index 2)
```

##### `ranges(filter=None) -> XAiWorkRangeList`

Returns named ranges in the table.

##### `sort(by_column, in_rows=None, direction=SortDirection.ASCENDING) -> XAiWorkTable`

Sorts the table by the specified column.

**Parameters:**
- `by_column` (`XAiWorkColumn`) - Column to sort by
- `in_rows` (`list[XAiWorkRow] | XAiWorkRowList | None`) - Rows to include in sort (None = all rows)
- `direction` (`SortDirection`) - ASCENDING or DESCENDING

**Returns:** The sorted table

**Example:**
```python
# Sort by column A ascending
table.sort(table.columns()[0], direction=XAiWorkApplication.SortDirection.ASCENDING)

# Sort by column B descending
table.sort(table.columns()[1], direction=XAiWorkApplication.SortDirection.DESCENDING)
```

---

### XAiWorkRange

**Bases:** `XAObject`

Represents a range of cells in a table.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Range coordinates (e.g., "A1:C5") (read/write) |
| `font_name` | `str` | Font of cells in range (read/write) |
| `font_size` | `float` | Font size of cells (read/write) |
| `format` | `CellFormat` | Cell format (read/write) |
| `alignment` | `Alignment` | Horizontal alignment (read/write) |
| `vertical_alignment` | `Alignment` | Vertical alignment (read/write) |
| `text_color` | `XAColor` | Text color (read/write) |
| `background_color` | `XAColor` | Background color (read/write) |
| `text_wrap` | `bool` | Whether text wraps (read/write) |
| `properties` | `dict` | All range properties |

#### Methods

##### `clear() -> XAiWorkRange`

Clears the content of every cell in the range.

**Returns:** The range (for method chaining)

**Example:**
```python
# Clear a specific range
table.cell_range.clear()

# Clear specific cells
table.rows()[0].clear()
```

##### `merge() -> XAiWorkRange`

Merges all cells in the range into one cell.

**Returns:** The range

**Example:**
```python
# Merge header cells
header_range = table.rows()[0]
header_range.merge()
```

##### `unmerge() -> XAiWorkRange`

Unmerges previously merged cells.

**Returns:** The range

##### `cells(filter=None) -> XAiWorkCellList`

Returns cells within the range.

##### `rows(filter=None) -> XAiWorkRowList`

Returns rows within the range.

##### `columns(filter=None) -> XAiWorkColumnList`

Returns columns within the range.

---

### XAiWorkRow

**Bases:** `XAiWorkRange`

Represents a single row in a table. Inherits all properties and methods from `XAiWorkRange`.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `address` | `int` | Row index (1-based, read-only) |
| `height` | `float` | Row height in pixels (read/write) |

**Example:**
```python
row = table.rows()[0]
print(f"Row {row.address} height: {row.height}")
row.height = 30  # Set row height
```

---

### XAiWorkColumn

**Bases:** `XAiWorkRange`

Represents a single column in a table. Inherits all properties and methods from `XAiWorkRange`.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `address` | `int` | Column index (1-based, read-only) |
| `width` | `float` | Column width in pixels (read/write) |

**Example:**
```python
col = table.columns()[0]
print(f"Column {col.address} width: {col.width}")
col.width = 150  # Set column width
```

---

### XAiWorkCell

**Bases:** `XAiWorkRange`

Represents a single cell in a table. Inherits all properties and methods from `XAiWorkRange`.

#### Properties

| Property | Type | Description |
|----------|------|-------------|
| `value` | `int | float | datetime | str | bool | None` | Cell value (read/write) |
| `formatted_value` | `str` | Formatted display value (read-only) |
| `formula` | `str` | Cell formula as text (read-only) |
| `row` | `XAiWorkRow` | Cell's containing row (read-only) |
| `column` | `XAiWorkColumn` | Cell's containing column (read-only) |

**Example:**
```python
cell = table.cells()[0]

# Read value
print(cell.value)
print(cell.formatted_value)

# Write value
cell.value = 42
cell.value = "Hello"
cell.value = 3.14

# Check for formula
if cell.formula:
    print(f"Formula: {cell.formula}")
```

---

### XAiWorkChart

**Bases:** `XAiWorkiWorkItem`

Represents a chart in a Numbers sheet. Charts are created from table data.

---

## List Classes

PyXA provides list wrapper classes with fast enumeration and bulk property access.

### XANumbersDocumentList

Bulk methods for document lists:

```python
docs = numbers.documents()
docs.name()                  # -> list[str]
docs.modified()              # -> list[bool]
docs.password_protected()    # -> list[bool]
docs.active_sheet()          # -> XANumbersSheetList
docs.document_template()     # -> XANumbersTemplateList
docs.properties()            # -> list[dict]
```

Filter methods:
```python
docs.by_name("Budget")
docs.by_modified(True)
docs.by_active_sheet(sheet)
docs.by_document_template(template)
docs.by_properties({"name": "Budget"})
```

### XANumbersSheetList

Bulk methods for sheet lists:

```python
sheets = doc.sheets()
sheets.name()        # -> list[str]
sheets.properties()  # -> list[dict]
```

Filter methods:
```python
sheets.by_name("Summary")
sheets.by_properties({"name": "Summary"})
```

### XANumbersTemplateList

```python
templates = numbers.templates()
templates.id()       # -> list[str]
templates.name()     # -> list[str]
templates.by_name("Invoice")
templates.by_id("template-id")
```

### XAiWorkTableList

```python
tables = sheet.tables()
tables.name()                # -> list[str]
tables.row_count()           # -> list[int]
tables.column_count()        # -> list[int]
tables.header_row_count()    # -> list[int]
tables.header_column_count() # -> list[int]
tables.footer_row_count()    # -> list[int]
tables.cell_range()          # -> XAiWorkRangeList
tables.selection_range()     # -> XAiWorkRangeList
```

Filter methods:
```python
tables.by_name("Sales Data")
tables.by_row_count(100)
tables.by_column_count(10)
```

### XAiWorkRangeList

```python
ranges = table.ranges()
ranges.name()              # -> list[str]
ranges.font_name()         # -> list[str]
ranges.font_size()         # -> list[float]
ranges.format()            # -> list[CellFormat]
ranges.alignment()         # -> list[Alignment]
ranges.text_color()        # -> list[XAColor]
ranges.background_color()  # -> list[XAColor]
ranges.text_wrap()         # -> list[bool]
ranges.vertical_alignment() # -> list[Alignment]
```

Filter methods:
```python
ranges.by_name("A1:B5")
ranges.by_font_name("Helvetica")
ranges.by_format(CellFormat.CURRENCY)
```

### XAiWorkRowList

```python
rows = table.rows()
rows.address()   # -> list[int]
rows.height()    # -> list[float]
```

Filter methods:
```python
rows.by_address(5)    # Get row 5
rows.by_height(30)    # Get rows with height 30
```

### XAiWorkColumnList

```python
columns = table.columns()
columns.address()  # -> list[int]
columns.width()    # -> list[float]
```

Filter methods:
```python
columns.by_address(3)   # Get column C (index 3)
columns.by_width(150)   # Get columns with width 150
```

### XAiWorkCellList

```python
cells = table.cells()
cells.value()            # -> list[Any]
cells.formatted_value()  # -> list[str]
cells.formula()          # -> list[str]
cells.row()              # -> XAiWorkRowList
cells.column()           # -> XAiWorkColumnList
```

Filter methods:
```python
cells.by_value(100)
cells.by_formatted_value("$100.00")
cells.by_formula("=SUM(A1:A10)")
cells.by_row(row)
cells.by_column(column)
```

---

## Enumerations

### ExportFormat

Export format options for Numbers documents.

| Value | OSType | Description |
|-------|--------|-------------|
| `NUMBERS` | `Nuff` | Native Numbers format (.numbers) |
| `PDF` | `Npdf` | PDF document |
| `MICROSOFT_EXCEL` | `Nexl` | Excel format (.xlsx) |
| `CSV` | `Ncsv` | Comma-separated values |
| `NUMBERS_09` | `Nnmb` | Numbers '09 format (legacy) |

**Example:**
```python
from PyXA.apps.Numbers import XANumbersApplication

doc.export("/path/output.pdf", XANumbersApplication.ExportFormat.PDF)
doc.export("/path/output.xlsx", XANumbersApplication.ExportFormat.MICROSOFT_EXCEL)
doc.export("/path/output.csv", XANumbersApplication.ExportFormat.CSV)
```

### ObjectType

Creatable object types for the `make()` method.

| Value | Description |
|-------|-------------|
| `DOCUMENT` | Numbers document |
| `SHEET` | Sheet within a document |
| `TABLE` | Table within a sheet |
| `CHART` | Chart |
| `IMAGE` | Image |
| `SHAPE` | Shape |
| `LINE` | Line |
| `TEXT_ITEM` | Text item |
| `AUDIO_CLIP` | Audio clip |
| `MOVIE` | Movie/video |
| `GROUP` | Group of items |
| `IWORK_ITEM` | Generic iWork item |

### CellFormat (iWork Base)

Cell format options for table cells.

| Value | OSType | Description |
|-------|--------|-------------|
| `AUTO` | `faut` | Automatic formatting |
| `CHECKBOX` | `fcch` | Checkbox (boolean) |
| `CURRENCY` | `fcur` | Currency format |
| `DATE_AND_TIME` | `fdtm` | Date and time |
| `FRACTION` | `ffra` | Fraction display |
| `DECIMAL_NUMBER` | `nmbr` | Decimal number |
| `PERCENT` | `fper` | Percentage |
| `POPUP_MENU` | `fcpp` | Popup menu selection |
| `SCIENTIFIC` | `fsci` | Scientific notation |
| `SLIDER` | `fcsl` | Slider control |
| `STEPPER` | `fcst` | Stepper control |
| `TEXT` | `ctxt` | Plain text |
| `DURATION` | `fdur` | Duration |
| `RATING` | `frat` | Star rating |
| `NUMERAL_SYSTEM` | `fcns` | Numeral system |

**Example:**
```python
from PyXA.apps.iWorkApplicationBase import XAiWorkApplication

cell.format = XAiWorkApplication.CellFormat.CURRENCY
cell.format = XAiWorkApplication.CellFormat.PERCENT
```

### Alignment (iWork Base)

Alignment options for cell content.

| Value | OSType | Description |
|-------|--------|-------------|
| `AUTO` | `aaut` | Automatic alignment |
| `LEFT` | `alft` | Left aligned |
| `CENTER_HORIZONTAL` | `actr` | Center aligned (horizontal) |
| `RIGHT` | `arit` | Right aligned |
| `JUSTIFY` | `ajst` | Justified |
| `TOP` | `avtp` | Top aligned (vertical) |
| `CENTER_VERTICAL` | `actr` | Center aligned (vertical) |
| `BOTTOM` | `avbt` | Bottom aligned |

**Example:**
```python
cell.alignment = XAiWorkApplication.Alignment.CENTER_HORIZONTAL
cell.vertical_alignment = XAiWorkApplication.Alignment.TOP
```

### SortDirection (iWork Base)

Sort direction options.

| Value | OSType | Description |
|-------|--------|-------------|
| `ASCENDING` | `ascn` | Sort A to Z, 0 to 9 |
| `DESCENDING` | `dscn` | Sort Z to A, 9 to 0 |

**Example:**
```python
table.sort(table.columns()[0], direction=XAiWorkApplication.SortDirection.DESCENDING)
```

---

## Quick Reference Tables

### Common Operations

| Task | Code |
|------|------|
| Get Numbers app | `numbers = PyXA.Application("Numbers")` |
| Create document | `doc = numbers.new_document()` |
| Create from template | `doc = numbers.new_document(template=numbers.templates().by_name("Invoice"))` |
| Get first document | `doc = numbers.documents()[0]` |
| Get all sheets | `sheets = doc.sheets()` |
| Get active sheet | `sheet = doc.active_sheet` |
| Set active sheet | `doc.active_sheet = doc.sheets()[1]` |
| Add new sheet | `sheet = doc.new_sheet({"name": "Data"})` |
| Get tables in sheet | `tables = sheet.tables()` |
| Get table by name | `table = sheet.tables().by_name("Table 1")` |
| Export to PDF | `doc.export("/path.pdf", ExportFormat.PDF)` |
| Export to Excel | `doc.export("/path.xlsx", ExportFormat.MICROSOFT_EXCEL)` |
| Export to CSV | `doc.export("/path.csv", ExportFormat.CSV)` |
| Save document | `doc.save()` |

### Table Operations

| Task | Code |
|------|------|
| Get all cells | `cells = table.cells()` |
| Get all rows | `rows = table.rows()` |
| Get all columns | `columns = table.columns()` |
| Get cell by name | `cell = table.cells().by_name("A1")` |
| Get row by index | `row = table.rows()[0]` |
| Get column by index | `col = table.columns()[0]` |
| Get row count | `count = table.row_count` |
| Get column count | `count = table.column_count` |
| Set row count | `table.row_count = 20` |
| Set column count | `table.column_count = 10` |
| Clear table | `table.cell_range.clear()` |
| Sort ascending | `table.sort(table.columns()[0])` |
| Sort descending | `table.sort(table.columns()[0], direction=SortDirection.DESCENDING)` |

### Cell Operations

| Task | Code |
|------|------|
| Read cell value | `value = cell.value` |
| Write cell value | `cell.value = 42` |
| Get formatted value | `display = cell.formatted_value` |
| Get cell formula | `formula = cell.formula` |
| Set cell format | `cell.format = CellFormat.CURRENCY` |
| Set font | `cell.font_name = "Helvetica"` |
| Set font size | `cell.font_size = 14` |
| Set alignment | `cell.alignment = Alignment.CENTER_HORIZONTAL` |
| Set text color | `cell.text_color = XAColor.red()` |
| Set background | `cell.background_color = XAColor.yellow()` |
| Enable text wrap | `cell.text_wrap = True` |

### Row/Column Operations

| Task | Code |
|------|------|
| Get row height | `height = row.height` |
| Set row height | `row.height = 30` |
| Get column width | `width = column.width` |
| Set column width | `column.width = 150` |
| Get row address | `index = row.address` |
| Get column address | `index = column.address` |
| Clear row | `row.clear()` |
| Clear column | `column.clear()` |
| Merge cells in row | `row.merge()` |

### Range Operations

| Task | Code |
|------|------|
| Get all cells range | `range = table.cell_range` |
| Get selection | `range = table.selection_range` |
| Clear range | `range.clear()` |
| Merge range | `range.merge()` |
| Unmerge range | `range.unmerge()` |
| Set range font | `range.font_name = "Arial"` |
| Set range format | `range.format = CellFormat.PERCENT` |

### Property Access Patterns

```python
# Single object property access
table = sheet.tables()[0]
print(table.name)
print(table.row_count)

# Bulk property access on lists
tables = sheet.tables()
print(tables.name())       # Returns list[str]
print(tables.row_count())  # Returns list[int]

# Filtering lists
large_tables = tables.greater_than("row_count", 100)
currency_cells = cells.by_format(CellFormat.CURRENCY)
```

### Reading Table Data

```python
# Read entire table as 2D array
table = sheet.tables()[0]
data = []
for row in table.rows():
    row_data = [cell.value for cell in row.cells()]
    data.append(row_data)

# Read specific column values
col_values = [cell.value for cell in table.columns()[0].cells()]

# Read specific row values
row_values = [cell.value for cell in table.rows()[0].cells()]
```

### Writing Table Data

```python
# Write to specific cell
table.cells().by_name("A1").value = "Header"

# Write row of data
row = table.rows()[0]
values = ["Name", "Age", "City"]
for i, cell in enumerate(row.cells()):
    if i < len(values):
        cell.value = values[i]

# Write 2D data array
data = [
    ["Name", "Score"],
    ["Alice", 95],
    ["Bob", 87]
]
for i, row_data in enumerate(data):
    row = table.rows()[i]
    for j, value in enumerate(row_data):
        row.cells()[j].value = value
```

---

## See Also

- [PyXA Numbers Documentation](https://skaplanofficial.github.io/PyXA/reference/apps/numbers.html) - Official PyXA documentation
- [PyXA iWork Base Documentation](https://github.com/SKaplanOfficial/PyXA/blob/main/PyXA/apps/iWorkApplicationBase.py) - Source for table/cell classes
- [numbers-basics.md](numbers-basics.md) - JXA fundamentals
- [numbers-recipes.md](numbers-recipes.md) - Common automation patterns
- [numbers-advanced.md](numbers-advanced.md) - Advanced techniques
