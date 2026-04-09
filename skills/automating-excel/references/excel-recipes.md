# Excel JXA Recipes

## The "List of Lists" Architecture (Critical)

Excel ranges are 2D matrices. JXA **strictly enforces** this structure - the most common source of "Type Mismatch" errors.

### 2D Array Rules

| Data Shape | JXA Format | Example |
|------------|------------|---------|
| Block (rows x cols) | `[[r1c1, r1c2], [r2c1, r2c2]]` | A1:B2 |
| Single row | `[[a, b, c]]` | Outer array wraps row |
| Single column | `[[a], [b], [c]]` | Each cell is a row |

### Writing Data (One-Shot Pattern)

```javascript
// Build 2D array in JavaScript
var data = [
    ["Name", "Score", "Grade"],
    ["Ada", 98, "A"],
    ["Lin", 95, "A"],
    ["Max", 87, "B"]
];

// Calculate range size
var rowCount = data.length;
var colCount = data[0].length;

// Define exact range
var startCell = ws.cells["A1"];
var targetRange = startCell.resize({ rowSize: rowCount, columnSize: colCount });

// One-shot write (single Apple Event)
targetRange.value = data;
```

### Reading Data

```javascript
// Read entire used range
var used = ws.usedRange;
var values = used.value();  // Returns 2D array

// Process in JavaScript (fast - local memory)
var filtered = values.filter(row => row[1] > 90);
```

## Performance: The One-Shot Rule

Each Apple Event has IPC latency. Cell-by-cell operations are **100-1000x slower** than batch operations.

### Bad (Slow) Pattern

```javascript
// DON'T DO THIS - 1000 Apple Events
for (var i = 0; i < 1000; i++) {
    ws.cells["A" + (i+1)].value = i;  // Slow!
}
```

### Good (Fast) Pattern

```javascript
// DO THIS - 1 Apple Event
var data = [];
for (var i = 0; i < 1000; i++) {
    data.push([i]);  // Build in memory
}
ws.ranges["A1:A1000"].value = data;  // Single write
```

## Formatting

### Cell Backgrounds

```javascript
var range = ws.ranges["A1:D1"];

// Using RGB array (modern)
range.interiorObject.color = [30, 50, 120];  // Dark blue

// Using colorIndex (legacy 56-color palette)
range.interiorObject.colorIndex = 3;  // Red
```

### Font Styling

```javascript
var header = ws.ranges["A1:D1"];
header.fontObject.name = "Arial";
header.fontObject.bold = true;
header.fontObject.size = 14;
header.fontObject.color = [255, 255, 255];  // White text
```

### Borders

```javascript
var range = ws.ranges["A1:D10"];
range.borders.lineStyle = "continuous";
range.borders.weight = "thin";
```

### Auto-fit Columns

```javascript
ws.columns["A:D"].autofit();
```

## Complete Report Generator

```javascript
function generateReport(outputPath, rawData) {
    var Excel = Application("Microsoft Excel");
    Excel.includeStandardAdditions = true;

    try {
        Excel.screenUpdating = false;
        Excel.displayAlerts = false;

        // Create workbook
        var wb = Excel.Workbooks.add();
        var ws = wb.worksheets[0];
        ws.name = "Report";

        // Prepare data with headers
        var headers = ["ID", "Name", "Value", "Status"];
        var dataMatrix = [headers].concat(rawData);

        // One-shot write
        var rowCount = dataMatrix.length;
        var colCount = dataMatrix[0].length;
        var range = ws.cells["A1"].resize({ rowSize: rowCount, columnSize: colCount });
        range.value = dataMatrix;

        // Format header row
        var headerRange = ws.ranges["A1:D1"];
        headerRange.interiorObject.color = [0, 51, 102];
        headerRange.fontObject.color = [255, 255, 255];
        headerRange.fontObject.bold = true;

        // Auto-fit and add borders
        ws.columns["A:D"].autofit();
        range.borders.lineStyle = "continuous";

        // Save
        wb.saveWorkbookAs({ filename: outputPath });

        console.log("Report generated: " + outputPath);

    } finally {
        Excel.screenUpdating = true;
        Excel.displayAlerts = true;
    }
}

// Usage
var testData = [
    [1, "Alpha", 100, "Active"],
    [2, "Beta", 200, "Pending"],
    [3, "Gamma", 150, "Active"]
];
generateReport("/Users/me/Desktop/Report.xlsx", testData);
```

## CSV Import Pattern

```javascript
function importCSV(csvPath, outputPath) {
    ObjC.import('Foundation');

    // Read CSV using ObjC (handles encoding properly)
    var nsPath = $(csvPath).stringByStandardizingPath;
    var content = $.NSString.stringWithContentsOfFileEncodingError(
        nsPath, $.NSUTF8StringEncoding, $()
    );
    var csvText = ObjC.unwrap(content);

    // Parse CSV to 2D array
    var rows = csvText.split('\n').filter(r => r.trim());
    var data = rows.map(row => row.split(','));

    // Write to Excel
    var Excel = Application("Microsoft Excel");
    var wb = Excel.Workbooks.add();
    var ws = wb.worksheets[0];

    var range = ws.cells["A1"].resize({
        rowSize: data.length,
        columnSize: data[0].length
    });
    range.value = data;

    wb.saveWorkbookAs({ filename: outputPath });
}
```

## Conditional Formatting (Manual)

JXA doesn't directly support conditional formatting rules, but you can apply formatting based on values:

```javascript
function applyConditionalColors(ws, dataRange) {
    var values = ws.ranges[dataRange].value();

    for (var r = 0; r < values.length; r++) {
        var value = values[r][0];  // Assuming first column has values
        var cellRef = "A" + (r + 1);

        if (value < 0) {
            ws.ranges[cellRef].fontObject.color = [255, 0, 0];  // Red
        } else if (value > 100) {
            ws.ranges[cellRef].fontObject.color = [0, 128, 0];  // Green
        }
    }
}
```
