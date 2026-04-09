# Keynote JXA Recipes

## Text Handling

Keynote text splits into two categories: **Placeholders** (from the master) and **Free Text** (added manually).

### Controlling Placeholders

Master slides expose title and body placeholders via convenience methods:

```javascript
var slide = doc.currentSlide;
var titleItem = slide.defaultTitleItem();
var bodyItem = slide.defaultBodyItem();

// Check existence (Blank slides have no title)
if (titleItem) {
    titleItem.objectText = "Automated Presentation";

    // Font styling
    titleItem.objectText.size = 64;
    titleItem.objectText.font = "Helvetica Neue-Bold";

    // Color is 16-bit RGB (0-65535)
    titleItem.objectText.color = [65535, 0, 0];  // Red
}

if (bodyItem) {
    bodyItem.objectText = "Generated content here";
}
```

### Creating Text Boxes

```javascript
var newTextBox = Keynote.TextItem({
    objectText: "Confidential Data",
    position: {x: 50, y: 50},
    width: 300,
    height: 100
});
slide.textItems.push(newTextBox);
```

## Shapes and Styling

Shapes are containers that can hold text and have fills/strokes.

### Writable Properties

| Property | Type | Description |
|----------|------|-------------|
| `position` | `{x, y}` | Top-left coordinates |
| `width`, `height` | Number | Dimensions |
| `rotation` | Number | Degrees |
| `opacity` | 0-100 | Transparency |
| `locked` | Boolean | Prevent editing |

```javascript
var rectangle = Keynote.Shape({
    position: {x: 400, y: 300},
    width: 200,
    height: 150,
    opacity: 80
});
slide.shapes.push(rectangle);

// Add text inside the shape
rectangle.objectText = "Status: Green";
```

**Note**: Some properties like `backgroundFillType` and `reflection` are often read-only. Create shapes with the desired fill initially or use stored custom styles.

## Images and Asset Management

```javascript
var imgPath = Path("/Users/username/Assets/diagram.png");
var slideImage = Keynote.Image({
    file: imgPath,
    position: {x: 800, y: 300},
    width: 600
    // Height auto-scales proportionally
});
slide.images.push(slideImage);
```

**Security Note**: macOS sandboxing may restrict access to arbitrary locations. Prefer `~/Documents` or `~/Desktop` for reliable access.

## Tables

Tables are the most complex objects but offer high value for reporting.

### Creating Tables

```javascript
var dataTable = Keynote.Table({
    position: {x: 100, y: 200},
    width: 900,
    height: 500,
    rowCount: 5,
    columnCount: 4,
    headerRowCount: 1,
    headerColumnCount: 0
});
slide.tables.push(dataTable);
```

### Batch Population Pattern

```javascript
var tableRef = slide.tables[0];

var rawData = [
    ["Metric", "2024", "2025", "Delta"],
    ["Revenue", "1.2M", "1.5M", "+25%"],
    ["Cost", "500k", "450k", "-10%"],
    ["Users", "10k", "15k", "+50%"]
];

for (var r = 0; r < rawData.length; r++) {
    for (var c = 0; c < rawData[r].length; c++) {
        // Validate bounds
        if (r < tableRef.rowCount() && c < tableRef.columnCount()) {
            tableRef.rows[r].cells[c].value = rawData[r][c];
        }
    }
}
```

**Performance Note**: Each `cell.value = ...` is a separate Apple Event. For large tables (10x10+), consider using clipboard paste with TSV data for bulk operations.

## Export Patterns

### PDF Export

```javascript
var exportPath = Path("/Users/username/Desktop/FinalDeck.pdf");

doc.export({
    to: exportPath,
    as: 'PDF',
    exportStyle: 'IndividualSlides',  // vs 'Handouts'
    allStages: true,                   // Include each build step
    skippedSlides: false               // Include hidden slides
});
```

### PowerPoint Export

```javascript
doc.export({
    to: Path("/Users/username/Desktop/deck.pptx"),
    as: 'Microsoft PowerPoint',
    password: "optional-password"
});
```

### Supported Export Formats

| Format | Constant |
|--------|----------|
| PDF | `'PDF'` |
| PowerPoint | `'Microsoft PowerPoint'` |
| Video | `'QuickTime movie'` |
| HTML | `'HTML'` |
| Legacy | `'Keynote 09'` |

## Complete Report Generator

```javascript
function generateReport(outputPath, reportData) {
    var Keynote = Application('Keynote');
    Keynote.includeStandardAdditions = true;

    // Create document
    var doc = Keynote.Document({
        documentTheme: Keynote.themes["White"],
        width: 1920,
        height: 1080
    });
    Keynote.documents.push(doc);

    // Title slide
    var titleSlide = doc.slides[0];
    var today = new Date().toLocaleDateString();
    titleSlide.defaultTitleItem().objectText = "Daily Report";
    titleSlide.defaultBodyItem().objectText = "Generated: " + today;

    // Data slide
    var blankMaster = doc.masterSlides["Blank"];
    var dataSlide = Keynote.Slide({ baseSlide: blankMaster });
    doc.slides.push(dataSlide);

    // Add table
    var table = Keynote.Table({
        position: {x: 100, y: 150},
        width: 800,
        height: 600,
        rowCount: reportData.length,
        columnCount: reportData[0].length
    });
    dataSlide.tables.push(table);

    // Populate
    var tableRef = dataSlide.tables[0];
    for (var r = 0; r < reportData.length; r++) {
        for (var c = 0; c < reportData[r].length; c++) {
            tableRef.rows[r].cells[c].value = reportData[r][c];
        }
    }

    // Export
    doc.export({
        to: Path(outputPath),
        as: "PDF",
        allStages: true
    });

    return { success: true, path: outputPath };
}

// Usage
var data = [
    ["Service", "Status", "Uptime"],
    ["API", "Green", "99.9%"],
    ["Database", "Green", "100%"],
    ["Auth", "Yellow", "99.5%"]
];
generateReport("/Users/me/Desktop/Report.pdf", data);
```
