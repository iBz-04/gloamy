# Pages JXA advanced patterns

## Image Insertion (ObjC Pasteboard Method)

**IMPORTANT**: Unlike Keynote, Pages does **NOT** support direct image insertion via a constructor like:
```javascript
// THIS DOES NOT WORK IN PAGES!
Pages.Image({ file: Path("/path/to/image.png"), position: {x: 100, y: 100} });
```

The Pages `image` class requires `image binary data` which cannot be easily instantiated from files.

### Recommended: ObjC Pasteboard Method

The most reliable way to insert images into Pages is using ObjC bridging with `NSImage` and `NSPasteboard`:

```javascript
ObjC.import('AppKit');

/**
 * Insert an image into Pages at current cursor position.
 * @param {string} imagePath - Absolute path to image file (PNG, JPEG, TIFF)
 */
function pasteImageFromFile(imagePath) {
    // Load image using NSImage
    const nsImage = $.NSImage.alloc.initWithContentsOfFile(imagePath);
    if (!nsImage) {
        throw new Error("Failed to load image: " + imagePath);
    }

    // Get pasteboard and clear it
    const pasteboard = $.NSPasteboard.generalPasteboard;
    pasteboard.clearContents;

    // Set TIFF data to pasteboard (universal format Pages accepts)
    pasteboard.setDataForType(nsImage.TIFFRepresentation, $.NSPasteboardTypeTIFF);

    // Paste into Pages
    const Pages = Application('Pages');
    Pages.activate();
    delay(0.3);

    // Simulate Cmd+V
    const se = Application('System Events');
    se.keystroke('v', { using: 'command down' });
    delay(1);
}

// Usage
pasteImageFromFile("/Users/you/images/diagram.png");
```

### Complete Example: Insert Multiple Images

```javascript
#!/usr/bin/env osascript -l JavaScript
'use strict';
ObjC.import('AppKit');

const Pages = Application('Pages');
Pages.includeStandardAdditions = true;

const images = [
    "/path/to/image1.png",
    "/path/to/image2.png"
];

function run() {
    Pages.activate();
    delay(1);

    const doc = Pages.documents[0]; // Use front document

    // Move to end of document
    const se = Application('System Events');
    se.keystroke('e', { using: ['command down', 'shift down'] });
    delay(0.3);
    se.keyCode(124); // Right arrow to deselect
    se.keystroke('\r'); // New line

    // Insert each image
    for (const imgPath of images) {
        const nsImage = $.NSImage.alloc.initWithContentsOfFile(imgPath);
        const pb = $.NSPasteboard.generalPasteboard;
        pb.clearContents;
        pb.setDataForType(nsImage.TIFFRepresentation, $.NSPasteboardTypeTIFF);

        Pages.activate();
        delay(0.3);
        se.keystroke('v', { using: 'command down' });
        delay(1);
        se.keystroke('\r');
        se.keystroke('\r');
    }

    doc.save();
    return "Images inserted successfully";
}
```

### Why This Works

1. **ObjC.import('AppKit')** - Enables access to macOS AppKit framework
2. **NSImage** - Native macOS image class that can load any image format
3. **NSPasteboard** - System pasteboard for copy/paste operations
4. **TIFF format** - Universal bitmap format that Pages reliably accepts
5. **System Events keystroke** - Simulates user pressing Cmd+V

### Supported Image Formats

NSImage supports: PNG, JPEG, TIFF, GIF, BMP, PDF, EPS, and more.

### Limitations

- Images are inserted at cursor position (no direct x,y positioning)
- Requires System Events accessibility permission
- Images default to text-wrap mode; adjust manually if needed
- Cannot insert images directly into table cells

---

## AppleScript bridge for gaps

For features not in JXA dictionary, use the AppleScript bridge:

```javascript
function runAppleScript(code) {
    const app = Application.currentApplication();
    app.includeStandardAdditions = true;
    return app.runScript(code, { language: "AppleScript" });
}

const as = `
  tell application "Pages"
    tell front document
      make new table with properties {column count:4, row count:10}
    end tell
  end tell
`;
runAppleScript(as);
```

---

## Table population (pre-made template)

1. Create a template document with a table
2. Open template, select the table, then write cell values:

```javascript
const Pages = Application('Pages');
const doc = Pages.documents[0];
const table = doc.tables[0];

// Set cell values
table.cells["A1"].value = "Header 1";
table.cells["B1"].value = "Header 2";
table.cells["A2"].value = "Data 1";
table.cells["B2"].value = "Data 2";
```

---

## Keynote vs Pages Image Comparison

| Feature | Keynote | Pages |
|---------|---------|-------|
| Direct file insertion | ✅ `Keynote.Image({file: Path(...)})` | ❌ Not supported |
| Position on creation | ✅ `position: {x, y}` | ❌ Not supported |
| Width/Height on creation | ✅ `width: 800` | ❌ Not supported |
| ObjC Pasteboard method | ✅ Works | ✅ Works (only option) |
| Binary data insertion | ⚠️ Complex | ⚠️ Complex |

**Recommendation**: For Pages, always use the ObjC Pasteboard method shown above.
