# Word JXA Basics

## Runtime Architecture

JXA scripts run in a separate process from Word. Communication occurs via Apple Events (IPC). When you access `wordApp.documents`, JXA creates an **Object Specifier** (a query reference), not the actual data. Data is only fetched when properties are explicitly accessed.

## Bootstrapping

```javascript
'use strict';

// Initialize the proxy for Microsoft Word
const word = Application("Microsoft Word");
word.includeStandardAdditions = true;

// Bring to front
word.activate();

// Access the name property triggers an Apple Event
const appName = word.name();
```

## Global Settings for Performance

Before intensive operations, disable UI updates:

```javascript
try {
    word.screenUpdating = false;  // Freeze UI during batch operations
    word.displayAlerts = false;   // Suppress confirmation dialogs

    // ... perform automation tasks ...

} finally {
    // ALWAYS restore settings
    word.screenUpdating = true;
    word.displayAlerts = true;
}
```

## Document Lifecycle

### Creating Documents

```javascript
// Create new document using make command
const newDoc = word.make({
    new: 'document',
    withProperties: {
        // Initial properties can be set here
    }
});
```

### Opening Documents

```javascript
// Open existing document - use absolute POSIX path
const doc = word.open("/Users/you/Documents/Report.docx");
```

### Saving Documents

```javascript
// Save to existing location
doc.save();

// Save As with specific format
doc.saveAs({
    fileName: "/Users/you/Desktop/NewReport.docx",
    fileFormat: 16  // wdFormatDocx
});
```

### Closing Documents

```javascript
doc.close({
    saving: 'no'  // or 'yes' to save changes
});
```

## File Format Enumerations

| Constant | Value | Description |
|----------|-------|-------------|
| wdFormatDocument | 16 | Default .docx |
| wdFormatPDF | 17 | PDF export |
| wdFormatText | 2 | Plain text |
| wdFormatRTF | 6 | Rich Text Format |
| wdFormatHTML | 8 | HTML format |
| wdFormatFlatXML | 19 | Flat XML |

## Getting Document Content

```javascript
// Get full document text
const fullText = doc.content.content();

// Access specific paragraph
const firstPara = doc.paragraphs[0].textObject.content();

// Count paragraphs
const paraCount = doc.paragraphs.length;
```

## The Object Specifier Model

```javascript
// This does NOT fetch data - creates a reference
const sheet = workbook.sheets[0];

// This DOES fetch data - triggers Apple Event
const sheetName = sheet.name();

// To inspect object properties
const props = doc.properties();
console.log(JSON.stringify(props));
```
