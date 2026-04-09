# Excel JXA Basics

## Runtime Architecture

JXA controls Excel through Apple Events (IPC). Unlike VBA which runs inside Excel's process, JXA scripts run externally. This means:

- Each property access/command is an IPC round-trip
- Objects are **Object Specifiers** (queries) not actual data
- Data is fetched only when explicitly accessed with `()` or `.value()`

## Bootstrapping

```javascript
'use strict';

var Excel = Application("Microsoft Excel");
Excel.includeStandardAdditions = true;

// Verify Excel is running
if (!Excel.running()) {
    Excel.activate();
    delay(1); // Allow launch time
}
```

## Application Object

The `Application('Microsoft Excel')` object is the root of the automation hierarchy:

- `Excel.frontmost` - Boolean for active window status
- `Excel.version` - Excel version string
- `Excel.activeWorkbook` - Currently focused workbook
- `Excel.workbooks` - Collection of all open workbooks

## Workbook Management

### Creating Workbooks

```javascript
// Create new workbook
var wb = Excel.Workbooks.add();

// Create from template
var templateWb = Excel.Workbooks.add({ template: "TemplateName" });
```

### Opening Workbooks

```javascript
// Use POSIX path with parameter object
var filePath = "/Users/Me/Documents/Data.xlsx";
var wb = Excel.openWorkbook({ workbookFileName: filePath });
```

### Saving Workbooks

```javascript
// Save to existing location
wb.save();

// Save As with format
wb.saveWorkbookAs({
    filename: "/Users/Me/Desktop/Report.xlsx",
    fileFormat: "xlsx"
});
```

## Worksheet Operations

### Accessing Worksheets

```javascript
// By name (preferred - robust)
var sheet = wb.worksheets["Data"];

// By index (fragile - order can change)
var firstSheet = wb.worksheets[0];

// Iterate all sheets
for (var i = 0; i < wb.worksheets.length; i++) {
    var currentSheet = wb.worksheets[i];
    console.log(currentSheet.name());
}
```

### Creating Worksheets

```javascript
var newSheet = Excel.Sheet({ name: "Analysis" });
wb.worksheets.push(newSheet);
```

### Worksheet Properties

```javascript
ws.name = "Data";           // Rename
ws.activate();              // Bring to front
ws.delete();                // Remove (displayAlerts=false needed)
```

## Range Addressing

Excel provides multiple ways to reference ranges:

```javascript
// A1 notation
sheet.ranges["A1:C10"]

// Single cell
sheet.cells["A1"]

// Row access
sheet.rows[1]  // Second row (0-indexed)

// Column access
sheet.columns["C"]

// Dynamic used range
sheet.usedRange  // Auto-adjusts to data
```

## Reading Cell Values

```javascript
// Single cell - returns scalar
var value = ws.ranges["A1"].value();

// Range - returns 2D array (List of Lists)
var data = ws.ranges["A1:C3"].value();
// Result: [[a1, b1, c1], [a2, b2, c2], [a3, b3, c3]]

// Used range - dynamic
var allData = ws.usedRange.value();
```

## Path Handling Notes

- Always use absolute POSIX paths: `/Users/name/file.xlsx`
- Use `saveWorkbookAs({ filename: "..." })` for save operations
- The `Path()` constructor may be needed in some contexts
- Tilde expansion (`~/`) may not work - use full paths
