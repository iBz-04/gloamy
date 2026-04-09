# Excel JXA Advanced Patterns

## Performance Optimization

### Global Settings (Always Restore!)

```javascript
var Excel = Application("Microsoft Excel");

// Store original state
var prevScreen = Excel.screenUpdating();
var prevAlerts = Excel.displayAlerts();
var prevCalc = Excel.calculation();

try {
    // Disable for performance
    Excel.screenUpdating = false;   // Freeze UI
    Excel.displayAlerts = false;    // Suppress dialogs
    Excel.calculation = -4135;      // xlCalculationManual

    // === Heavy operations here ===

} finally {
    // ALWAYS restore state
    Excel.screenUpdating = prevScreen;
    Excel.displayAlerts = prevAlerts;
    Excel.calculation = prevCalc;
}
```

### Calculation Constants

| Constant | Value | Description |
|----------|-------|-------------|
| xlCalculationAutomatic | -4105 | Auto recalculate |
| xlCalculationManual | -4135 | Manual only |
| xlCalculationSemiautomatic | 2 | Auto except tables |

## The Objective-C Bridge

### File System Operations

```javascript
ObjC.import('Foundation');

function fileExists(path) {
    return $.NSFileManager.defaultManager
        .fileExistsAtPath($(path).stringByStandardizingPath);
}

function listDirectory(dirPath) {
    var fm = $.NSFileManager.defaultManager;
    var contents = fm.contentsOfDirectoryAtPathError(dirPath, $());
    return ObjC.unwrap(contents);
}

function getDesktopPath() {
    var paths = $.NSSearchPathForDirectoriesInDomains(
        $.NSDesktopDirectory,
        $.NSUserDomainMask,
        true
    );
    return ObjC.unwrap(paths.objectAtIndex(0));
}
```

### Reading Files with Proper Encoding

```javascript
function readTextFile(path) {
    var errorRef = $();
    var nsPath = $(path).stringByStandardizingPath;

    var content = $.NSString.stringWithContentsOfFileEncodingError(
        nsPath,
        $.NSUTF8StringEncoding,
        errorRef
    );

    if (errorRef.code) {
        throw new Error("Read failed: " + ObjC.unwrap(errorRef.localizedDescription));
    }

    return ObjC.unwrap(content);
}
```

## VBA Escape Hatch

When JXA lacks functionality, call existing VBA macros:

```javascript
// Execute a VBA macro
Excel.run("FormatReport");

// Execute with parameters
Excel.run("UpdateCalculation", { arg1: 500 });

// Call macro in specific workbook
var wb = Excel.workbooks["Report.xlsm"];
wb.runVBMacro("Module1.ProcessData");
```

**Use Cases:**
- Pivot Table creation (unstable in JXA)
- Complex chart configurations
- Conditional formatting rules
- Advanced filtering

## AppleScript to JXA Translation

### Element vs Property Rule

| AppleScript | JXA | Type |
|-------------|-----|------|
| `worksheet` element of workbook | `workbook.worksheets` | Collection |
| `display alerts` property | `Excel.displayAlerts` | Property |
| `active sheet` property | `Excel.activeSheet` | Property |

### Command Translation

| AppleScript | JXA |
|-------------|-----|
| `set x to value of range "A1"` | `var x = range.value()` |
| `set value of range "A1" to x` | `range.value = x` |
| `count of worksheets` | `wb.worksheets.length` |
| `make new workbook` | `Excel.Workbooks.add()` |
| `delete worksheet "Sheet1"` | `ws.delete()` |
| `save workbook as...` | `wb.saveWorkbookAs({...})` |

## 2D Array Rules (Critical)

JXA enforces strict array structure for ranges:

```javascript
// Single row: MUST wrap in outer array
range.value = [[a, b, c]];  // NOT [a, b, c]

// Single column: MUST wrap each value
range.value = [[a], [b], [c]];  // NOT [[a, b, c]]

// Block: Standard 2D array
range.value = [[r1c1, r1c2], [r2c1, r2c2]];

// Range dimensions MUST match array dimensions
// Mismatch causes Error -1700 (Type conversion failed)
```

## Debugging

### Common Error Codes

| Code | Meaning | Typical Cause |
|------|---------|---------------|
| -1700 | Type conversion failed | Array dimension mismatch |
| -1728 | Object not found | Invalid index or name |
| -10000 | Application not running | Excel not launched |
| -1708 | Event not handled | Invalid command |

### Inspecting Objects

```javascript
// Get all properties of an object
var props = ws.properties();
console.log(JSON.stringify(props, null, 2));

// List available methods (triggers heavy Apple Events)
console.log(Object.keys(ws));
```

### NSLog for Real-Time Debugging

```javascript
function log(msg) {
    $.NSLog($.NSString.alloc.initWithUTF8String(msg));
}

// Usage
log("Processing row: " + i);
```

## Batch Processing Pattern

```javascript
function batchProcess(inputDir, outputDir) {
    ObjC.import('Foundation');

    var Excel = Application("Microsoft Excel");
    var fm = $.NSFileManager.defaultManager;

    // List Excel files
    var contents = fm.contentsOfDirectoryAtPathError(inputDir, $());
    var files = ObjC.unwrap(contents).filter(f => f.endsWith('.xlsx'));

    try {
        Excel.screenUpdating = false;
        Excel.displayAlerts = false;

        files.forEach(function(file) {
            try {
                var inputPath = inputDir + "/" + file;
                var outputPath = outputDir + "/" + file.replace('.xlsx', '_processed.xlsx');

                var wb = Excel.openWorkbook({ workbookFileName: inputPath });

                // Process workbook...
                var ws = wb.worksheets[0];
                var data = ws.usedRange.value();
                // Transform data in JavaScript
                var processed = data.map(row => row.map(cell => cell * 2));
                ws.usedRange.value = processed;

                wb.saveWorkbookAs({ filename: outputPath });
                wb.close({ saving: 'no' });

                console.log("Processed: " + file);
            } catch (e) {
                console.log("Error processing " + file + ": " + e.message);
            }
        });

    } finally {
        Excel.screenUpdating = true;
        Excel.displayAlerts = true;
    }
}
```

## Production Template

```javascript
/**
 * Excel JXA Automation Controller
 * Production-grade template
 */
'use strict';

ObjC.import('Foundation');

function run() {
    var Excel = Application("Microsoft Excel");
    Excel.includeStandardAdditions = true;

    // Store original state
    var state = {
        screen: Excel.screenUpdating(),
        alerts: Excel.displayAlerts(),
        calc: Excel.calculation()
    };

    try {
        // Performance mode
        Excel.screenUpdating = false;
        Excel.displayAlerts = false;
        Excel.calculation = -4135;

        // === AUTOMATION LOGIC ===

        console.log("Completed successfully");

    } catch (error) {
        console.log("Error: " + error.message);
        Excel.displayAlerts = true;

    } finally {
        // Restore state
        Excel.screenUpdating = state.screen;
        Excel.displayAlerts = state.alerts;
        Excel.calculation = state.calc;
    }
}
```
