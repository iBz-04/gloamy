# Word JXA Recipes

## Text Manipulation with Range Objects

The Range object allows programmatic text manipulation without changing cursor position.

### Append Text via Range

```javascript
const range = doc.content;
range.collapse({ direction: 0 }); // 0 = collapse to end
range.insertAfter("Automated Appendix\n");
```

### Collapse Direction Enums

| Constant | Value | Description |
|----------|-------|-------------|
| wdCollapseStart | 1 | Collapse to beginning |
| wdCollapseEnd | 0 | Collapse to end |

### Insert Text at Specific Position

```javascript
// Insert after specific paragraph
const para = doc.paragraphs[2];
const range = para.textObject;
range.collapse({ direction: 0 });
range.insertAfter("\n[INSERTED SECTION]\n");
```

## Find and Replace

### Global Replace All

```javascript
function performGlobalReplace(doc, searchString, replaceString) {
    const range = doc.content;
    const findObj = range.find;

    // Clear previous settings
    findObj.clearFormatting();
    findObj.replacement.clearFormatting();

    // Set search criteria
    findObj.text = searchString;
    findObj.replacement.text = replaceString;
    findObj.forward = true;
    findObj.matchCase = false;
    findObj.matchWholeWord = true;
    findObj.matchWildcards = false;

    // Execute replace all
    findObj.execute({
        replace: 2,  // wdReplaceAll
        wrap: 1      // wdFindContinue
    });
}

// Usage: Template filling
performGlobalReplace(doc, "{{CLIENT}}", "Acme Corporation");
performGlobalReplace(doc, "{{DATE}}", new Date().toLocaleDateString());
```

### Find/Replace Enums

| Constant | Value | Description |
|----------|-------|-------------|
| wdReplaceNone | 0 | Search only |
| wdReplaceOne | 1 | Replace first |
| wdReplaceAll | 2 | Replace all |
| wdFindStop | 0 | Stop at end |
| wdFindContinue | 1 | Wrap around |
| wdFindAsk | 2 | Ask user |

## Tables

### Create and Populate Table

```javascript
function insertReportTable(doc, rowCount, colCount) {
    const range = doc.content;
    range.collapse({ direction: 0 }); // Append to end

    const table = doc.tables.add(range, rowCount, colCount);
    table.style = "Table Grid";
    return table;
}

// Populate cells
const table = insertReportTable(doc, 5, 4);
table.rows[0].cells[0].content = "Header 1";
table.rows[0].cells[1].content = "Header 2";
table.rows[1].cells[0].content = "Data 1";
```

### Fast Table Population (Tab-Delimited)

For large tables, avoid cell-by-cell Apple Events:

```javascript
function fastTableGeneration(doc, dataArray) {
    const range = doc.content;
    range.collapse({ direction: 0 });

    // Convert 2D array to tab-delimited string
    const flatText = dataArray.map(row => row.join('\t')).join('\r');
    range.insertAfter(flatText);

    // Then convert to table (much faster than cell-by-cell)
    // Note: May need to select and convert manually
}
```

## Formatting

### Direct Font Formatting

```javascript
const range = doc.paragraphs[0].textObject;
range.font.name = "Helvetica Neue";
range.font.size = 14;
range.font.bold = true;
range.font.colorIndex = 6;  // wdRed
```

### Apply Named Styles

```javascript
// Apply heading style
doc.paragraphs[0].style = "Heading 1";

// Apply body style
doc.paragraphs[1].style = "Normal";
```

**Warning**: Style names are localized. "Heading 1" may fail on non-English systems.

## PDF Export

```javascript
const WdSaveFormat = { PDF: 17, DOCX: 16 };

function saveAsPDF(doc, outputPath) {
    doc.saveAs({
        fileName: outputPath,
        fileFormat: WdSaveFormat.PDF
    });
}

// Usage
saveAsPDF(doc, "/Users/you/Desktop/Report.pdf");
```

## Template Filler Pattern

Complete workflow for document generation:

```javascript
function fillTemplate(templatePath, outputPath, replacements) {
    const word = Application("Microsoft Word");
    word.includeStandardAdditions = true;

    try {
        word.screenUpdating = false;
        word.displayAlerts = false;

        // Open template
        const doc = word.open(templatePath);

        // Apply all replacements
        for (const [placeholder, value] of Object.entries(replacements)) {
            performGlobalReplace(doc, placeholder, value);
        }

        // Save as PDF
        doc.saveAs({
            fileName: outputPath,
            fileFormat: 17  // PDF
        });

        doc.close({ saving: 'no' });

    } finally {
        word.screenUpdating = true;
        word.displayAlerts = true;
    }
}

// Usage
fillTemplate(
    "/Users/you/Templates/Invoice.docx",
    "/Users/you/Desktop/Invoice_001.pdf",
    {
        "{{CLIENT}}": "Acme Corp",
        "{{AMOUNT}}": "$5,000",
        "{{DATE}}": "January 14, 2025"
    }
);
```
