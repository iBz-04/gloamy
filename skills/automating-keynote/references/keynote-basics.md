# Keynote JXA Basics

## The Open Scripting Architecture (OSA)

Keynote exposes automation capabilities through an AppleScript dictionary (`.sdef` file). JXA dynamically maps these AppleScript definitions into JavaScript objects at runtime. You must read the AppleScript dictionary to understand capabilities, but write JXA for logic.

## Bootstrapping

```javascript
'use strict';

// Initialize the System Context
var app = Application.currentApplication();
app.includeStandardAdditions = true;

// Initialize Keynote
var Keynote = Application('Keynote');
Keynote.includeStandardAdditions = true;

// Verify Application State
if (!Keynote.running()) {
    Keynote.activate();
    delay(1); // Allow launch time
}
```

## Application Object

The `Application('Keynote')` object is the root of the automation hierarchy:

- `Keynote.frontmost` - Boolean for active window
- `Keynote.version` - Version string for feature checking
- `Keynote.selection` - Array of currently selected objects
- `Keynote.documents` - Collection of open presentations
- `Keynote.themes` - Collection of available themes

## Document Creation and Themes

A Keynote document must be born from a Theme. Attempting to create a document without a valid theme will fail.

### Validating Themes

```javascript
var themeName = "White";  // Standard theme
var selectedTheme = null;

try {
    selectedTheme = Keynote.themes[themeName];
} catch (e) {
    console.log("Theme not found, defaulting to Black");
    selectedTheme = Keynote.themes[0];
}
```

### Creating Documents

```javascript
var doc = Keynote.Document({
    documentTheme: selectedTheme,
    width: 1920,  // Optional: Force 1080p
    height: 1080
});

// The 'push' command triggers actual window creation
Keynote.documents.push(doc);
```

### Opening Documents

```javascript
// Use Path() object for file operations
var targetPath = Path("/Users/username/Documents/Presentation.key");
var openDoc = Keynote.open(targetPath);
```

### Saving Documents

```javascript
var saveDest = Path("/Users/username/Desktop/MyPresentation.key");
doc.save({ in: saveDest });

// Save existing document
doc.save();
```

## Master Slides

Every slide is an instance of a Master Slide. You cannot create a slide without defining its parent master.

```javascript
// List available masters
for (var i = 0; i < doc.masterSlides.length; i++) {
    console.log(doc.masterSlides[i].name());
}
```

### Common Master Slide Names

- "Title & Subtitle"
- "Title & Bullets"
- "Photo - Horizontal"
- "Blank"
- "Title - Center"
- "Title - Top"

**Note**: Names vary by theme. Always validate before use.

## Creating Slides

```javascript
// Get the master
var titleMaster = doc.masterSlides["Title & Subtitle"];

// Create slide with master
var slideOne = Keynote.Slide({
    baseSlide: titleMaster
});

// Add to document
doc.slides.push(slideOne);
```

## Navigation

The `currentSlide` property represents the slide visible in the editor:

```javascript
// Read current slide
var visibleSlide = doc.currentSlide();

// Jump to specific slide
doc.currentSlide = doc.slides[3];  // 4th slide (0-indexed)
```

## The iWork Item Model

All content on a slide inherits from `iWork item`:

| Class | Description |
|-------|-------------|
| `text item` | Text boxes, titles, bullets |
| `shape` | Rectangles, circles, stars |
| `image` | Imported bitmaps |
| `line` | Lines and arrows |
| `table` | Data tables |
| `chart` | Charts and graphs |
| `group` | Grouped objects |

## Controlling Placeholders

Master slides contain "Title" and "Body" placeholders:

```javascript
var slide = doc.currentSlide;
var titleItem = slide.defaultTitleItem();
var bodyItem = slide.defaultBodyItem();

// Check existence (Blank slides have no placeholders)
if (titleItem) {
    titleItem.objectText = "Automated Presentation";
}

if (bodyItem) {
    bodyItem.objectText = "Generated content here";
}
```

## Text Styling

```javascript
var titleItem = slide.defaultTitleItem();
if (titleItem) {
    titleItem.objectText = "Styled Title";

    // Font properties
    titleItem.objectText.size = 64;
    titleItem.objectText.font = "Helvetica Neue-Bold";

    // Color is 16-bit RGB (0-65535)
    titleItem.objectText.color = [65535, 0, 0];  // Red
}
```

## File Path Notes

- Always use `Path()` constructor for file operations
- Use absolute POSIX paths: `/Users/name/file.key`
- Sandboxing may restrict access to certain directories
- Prefer `~/Documents` or `~/Desktop` for safer access
