# Word JXA Advanced Patterns

## The Objective-C Bridge

JXA's `ObjC` bridge provides access to macOS frameworks, enabling functionality beyond Word's AppleScript dictionary.

### Import Foundation Framework

```javascript
ObjC.import('Foundation');
```

## File System Management with NSFileManager

### Check File Existence

```javascript
function fileExists(posixPath) {
    const fileManager = $.NSFileManager.defaultManager;
    const nsPath = $(posixPath).stringByStandardizingPath;
    return fileManager.fileExistsAtPath(nsPath);
}

// Usage - validate before opening
if (fileExists("/Users/you/Documents/Report.docx")) {
    const doc = word.open("/Users/you/Documents/Report.docx");
}
```

### Read Text Files with Proper Encoding

AppleScript's `read` command struggles with UTF-8. Use NSString:

```javascript
function readFileContent(path) {
    const errorRef = $();
    const nsPath = $(path).stringByStandardizingPath;

    const nsString = $.NSString.stringWithContentsOfFileEncodingError(
        nsPath,
        $.NSUTF8StringEncoding,
        errorRef
    );

    if (errorRef.code) {
        const errorDesc = ObjC.unwrap(errorRef.localizedDescription);
        throw new Error(`Failed to read: ${path}. ${errorDesc}`);
    }

    return ObjC.unwrap(nsString);
}
```

### Write Text Files

```javascript
function writeFileContent(path, content) {
    const nsString = $(content);
    const nsPath = $(path).stringByStandardizingPath;
    const errorRef = $();

    nsString.writeToFileAtomicallyEncodingError(
        nsPath,
        true,
        $.NSUTF8StringEncoding,
        errorRef
    );

    if (errorRef.code) {
        throw new Error(`Write failed: ${ObjC.unwrap(errorRef.localizedDescription)}`);
    }
}
```

## Clipboard Bridge for Images (Sandbox Workaround)

Word's `inlineShapes.addPicture()` often fails due to macOS sandboxing. Bypass via clipboard:

```javascript
ObjC.import('AppKit');

function loadImageToClipboard(imagePath) {
    const pasteboard = $.NSPasteboard.generalPasteboard;
    pasteboard.clearContents;

    const nsImage = $.NSImage.alloc.initWithContentsOfFile(imagePath);

    if (!nsImage || !nsImage.valid) {
        return false;
    }

    const array = $.NSArray.arrayWithObject(nsImage);
    pasteboard.writeObjects(array);
    return true;
}

// Usage: Load to clipboard, then paste in Word
function insertImageViaPaste(doc, imagePath) {
    if (loadImageToClipboard(imagePath)) {
        // Position cursor where image should go
        const range = doc.content;
        range.collapse({ direction: 0 });

        // Paste from clipboard
        range.paste();
    }
}
```

## Batch Processing Pattern

Process multiple documents efficiently:

```javascript
function batchConvertToPDF(inputDir, outputDir) {
    ObjC.import('Foundation');

    const word = Application("Microsoft Word");
    word.includeStandardAdditions = true;

    const fm = $.NSFileManager.defaultManager;
    const contents = fm.contentsOfDirectoryAtPathError(inputDir, $());

    // Filter for .docx files
    const files = ObjC.unwrap(contents).filter(f => f.endsWith('.docx'));

    try {
        word.screenUpdating = false;
        word.displayAlerts = false;

        for (const file of files) {
            try {
                const inputPath = `${inputDir}/${file}`;
                const outputPath = `${outputDir}/${file.replace('.docx', '.pdf')}`;

                const doc = word.open(inputPath);
                doc.saveAs({
                    fileName: outputPath,
                    fileFormat: 17  // PDF
                });
                doc.close({ saving: 'no' });

                console.log(`Converted: ${file}`);
            } catch (e) {
                console.log(`Error processing ${file}: ${e.message}`);
                // Continue with next file
            }
        }
    } finally {
        word.screenUpdating = true;
        word.displayAlerts = true;
    }
}
```

## Debugging and Error Handling

### Try/Catch Pattern

```javascript
try {
    word.open(path);
} catch (e) {
    console.log("Word Error: " + e.message);
    // e.message often contains Apple Event error number
}
```

### Common Error Codes

| Code | Meaning |
|------|---------|
| -1700 | Type conversion failed |
| -1728 | Object not found |
| -10000 | Application not running |
| -1708 | Event not handled |

### Logging with NSLog

For real-time debugging when console.log is buffered:

```javascript
function log(msg) {
    $.NSLog($.NSString.alloc.initWithUTF8String(msg));
}
```

## Calling VBA Macros from JXA

When JXA lacks functionality, call existing VBA:

```javascript
// Execute a VBA macro stored in the workbook
word.run("FormatReport");

// Execute with parameters
word.run("UpdateCalculation", { arg1: 500 });
```

## AppleScript Dictionary Translation Rules

When reading AppleScript documentation:

1. **Elements become Collections**: `worksheet` -> `workbook.worksheets`
2. **Properties become CamelCase**: `display alerts` -> `displayAlerts`
3. **Commands use parameter objects**: `save in folder` -> `save({ in: folder })`

### Mapping Table

| AppleScript | JXA |
|-------------|-----|
| `set x to value of range "A1"` | `var x = range.value()` |
| `set value of range "A1" to x` | `range.value = x` |
| `count of worksheets` | `wb.worksheets.length` |
| `make new document` | `word.make({ new: 'document' })` |
| `delete worksheet "Sheet1"` | `ws.delete()` |

## Production Script Template

```javascript
/**
 * Word Automation Controller
 * Production-grade template with proper error handling
 */
'use strict';

ObjC.import('Foundation');

function run() {
    const word = Application("Microsoft Word");
    word.includeStandardAdditions = true;

    // Store original state
    const originalScreenUpdating = word.screenUpdating;

    try {
        word.screenUpdating = false;
        word.displayAlerts = false;

        // === YOUR AUTOMATION LOGIC HERE ===

        console.log("Automation completed successfully");

    } catch (error) {
        console.log("Critical Error: " + error.message);
        // Re-enable alerts to show error
        word.displayAlerts = true;

    } finally {
        // ALWAYS restore state
        word.screenUpdating = true;
        word.displayAlerts = true;
    }
}
```
