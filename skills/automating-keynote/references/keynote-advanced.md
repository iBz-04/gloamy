# Keynote JXA Advanced Patterns

## The AppleScript Bridge for Charts

The `add chart` command in Keynote's dictionary requires specific AppleEvent descriptor types that JXA cannot serialize correctly. The solution: prepare data in JavaScript, execute via AppleScript.

### Why the Bridge is Necessary

```javascript
// THIS FAILS - JXA can't serialize arrays for add chart
Keynote.addChart({
    rowNames: ["Q1", "Q2"],  // Error: parameter missing
    columnNames: ["Sales"],
    data: [[50], [65]]
});
```

### The Bridge Pattern

```javascript
function createChartViaBridge(slideIndex, rowNames, colNames, dataRows) {
    // Convert JS arrays to AppleScript list syntax
    var asRows = '{' + rowNames.map(s => '"' + s + '"').join(',') + '}';
    var asCols = '{' + colNames.map(s => '"' + s + '"').join(',') + '}';
    var asData = '{' + dataRows.map(row => '{' + row.join(',') + '}').join(',') + '}';

    var asScript = `
        tell application "Keynote"
            tell document 1
                tell slide ${slideIndex}
                    add chart row names ${asRows} column names ${asCols} data ${asData} ¬
                    type vertical_bar_2d group by chart row
                end tell
            end tell
        end tell
    `;

    var app = Application.currentApplication();
    app.includeStandardAdditions = true;
    app.runScript(asScript);
}

// Usage
createChartViaBridge(3,
    ["Q1", "Q2", "Q3", "Q4"],
    ["Sales", "Marketing"],
    [[50, 30], [65, 40], [80, 50], [90, 60]]
);
```

### Chart Type Constants

| Type | Constant |
|------|----------|
| Vertical Bar | `vertical_bar_2d` |
| Horizontal Bar | `horizontal_bar_2d` |
| Pie | `pie_2d` |
| Line | `line_2d` |
| Scatter | `scatter_2d` |
| Bubble | `bubble_2d` |

**Note**: 3D variants exist but are less stable in scripting.

### Post-Creation Positioning

```javascript
// After chart creation, reposition via JXA
if (chartSlide.charts.length > 0) {
    chartSlide.charts[0].position = {x: 500, y: 200};
    chartSlide.charts[0].width = 900;
}
```

## Slide Transitions

Every slide has a `transitionProperties` record.

```javascript
var slide = doc.currentSlide;

slide.transitionProperties = {
    transitionEffect: "dissolve",
    transitionDuration: 1.5,      // Seconds
    transitionDelay: 0.5,         // Seconds
    automaticTransition: true     // Auto-advance
};
```

### Transition Effects

| Effect | Identifier |
|--------|------------|
| None | `"no transition"` |
| Magic Move | `"magic move"` |
| Dissolve | `"dissolve"` |
| Push | `"push"` |
| Reveal | `"reveal"` |
| Fade | `"fade through color"` |
| Iris | `"iris"` |
| Cube | `"cube"` |
| Flip | `"flop"` |
| Page Flip | `"page flip"` |

## Magic Move Animation

Magic Move creates smooth transitions between objects that share identity across slides.

### Implementation Strategy

1. **Compose Slide A**: Add objects
2. **Duplicate Slide A**: Creates Slide B with linked objects
3. **Modify Slide B**: Change position, opacity, scale, rotation
4. **Set Transition**: Apply "magic move" to Slide A

```javascript
// 1. Create base slide
var slideA = Keynote.Slide({ baseSlide: doc.masterSlides["Blank"] });
doc.slides.push(slideA);

var orb = Keynote.Shape({
    position: {x: 100, y: 500},
    width: 100,
    height: 100
});
slideA.shapes.push(orb);
orb.objectText = "Start";

// 2. Duplicate (objects maintain identity)
var slideB = slideA.duplicate();

// 3. Transform on destination slide
slideB.shapes[0].position = {x: 1500, y: 200};
slideB.shapes[0].opacity = 50;
slideB.shapes[0].rotation = 45;
slideB.shapes[0].objectText = "Finish";

// 4. Apply Magic Move
slideA.transitionProperties = {
    transitionEffect: "magic move",
    transitionDuration: 2.0
};
```

## GUI Scripting (Nuclear Option)

When the AppleScript dictionary lacks features (builds, inspector settings), use System Events.

### Prerequisites

- Enable "Accessibility" in System Preferences > Security & Privacy
- GUI scripting is slower and fragile (breaks if UI changes)

### The System Events Hierarchy

```
Application → Process → Window → Group → Toolbar/Splitter → Button/Menu
```

### Applying an "Anvil" Build Effect

Object builds (animations) are not exposed in the API. Automate the GUI instead:

```javascript
function applyAnvilBuild() {
    var sys = Application('System Events');
    var kProc = sys.processes['Keynote'];
    var frontWindow = kProc.windows[0];

    // Ensure Keynote is active
    Application('Keynote').activate();
    delay(0.3);

    // 1. Open Animate Inspector
    var toolbar = frontWindow.toolbars[0];
    var animateBtn = toolbar.radioGroups[0].radioButtons['Animate'];
    if (animateBtn.value() === 0) {
        animateBtn.click();
        delay(0.2);
    }

    // 2. Click 'Add an Effect'
    // Path varies by macOS version
    var inspector = frontWindow.groups[0];
    var addBtn = inspector.buttons['Add an Effect'];
    addBtn.click();
    delay(0.2);

    // 3. Type effect name and confirm
    sys.keystroke("Anvil");
    sys.keyCode(36);  // Return key
}
```

### Automating Menus

Menus are more stable than inspectors:

```javascript
var sys = Application('System Events');
var kProc = sys.processes['Keynote'];
var menuBar = kProc.menuBars[0];

// Format > Font > Bold
var formatMenu = menuBar.menus['Format'];
var fontSub = formatMenu.menuItems['Font'];
var boldItem = fontSub.menus[0].menuItems['Bold'];
boldItem.click();
```

## Debugging

### Common Error Codes

| Code | Meaning | Typical Cause |
|------|---------|---------------|
| -1700 | Type conversion failed | Array/object format mismatch |
| -1728 | Object not found | Invalid index or name |
| -10000 | Application not running | Keynote not launched |
| -1708 | Event not handled | Invalid command |

### NSLog for Real-Time Output

```javascript
ObjC.import('Foundation');

function log(msg) {
    $.NSLog($.NSString.alloc.initWithUTF8String(msg));
}

log("Processing slide: " + slideIndex);
```

## Production Template

```javascript
/**
 * Keynote Automation Controller
 * Production-grade template
 */
'use strict';

(function() {
    // 1. CONFIGURATION
    var app = Application.currentApplication();
    app.includeStandardAdditions = true;

    var Keynote = Application('Keynote');
    Keynote.includeStandardAdditions = true;

    // Ensure Keynote is ready
    if (!Keynote.running()) {
        Keynote.activate();
        delay(1);
    }

    try {
        // 2. DOCUMENT CREATION
        var doc = Keynote.Document({
            documentTheme: Keynote.themes["White"],
            width: 1920,
            height: 1080
        });
        Keynote.documents.push(doc);

        // === YOUR AUTOMATION LOGIC HERE ===

        // 3. EXPORT
        var exportPath = Path(
            app.pathTo("desktop").toString() + "/Output.pdf"
        );
        doc.export({
            to: exportPath,
            as: "PDF",
            allStages: true
        });

        // 4. NOTIFY USER
        app.displayNotification("Presentation generated", {
            withTitle: "Keynote Automation",
            subtitle: "PDF on Desktop"
        });

    } catch (error) {
        console.log("Error: " + error.message);

        app.displayAlert("Automation Failed", {
            message: error.message,
            as: "critical"
        });
    }
})();
```

## Batch Processing Pattern

```javascript
ObjC.import('Foundation');

function batchExportToPDF(inputDir, outputDir) {
    var Keynote = Application('Keynote');
    var fm = $.NSFileManager.defaultManager;

    // List .key files
    var contents = fm.contentsOfDirectoryAtPathError(inputDir, $());
    var files = ObjC.unwrap(contents).filter(f => f.endsWith('.key'));

    for (var i = 0; i < files.length; i++) {
        try {
            var inputPath = inputDir + "/" + files[i];
            var outputPath = outputDir + "/" + files[i].replace('.key', '.pdf');

            var doc = Keynote.open(Path(inputPath));

            doc.export({
                to: Path(outputPath),
                as: 'PDF'
            });

            doc.close({ saving: 'no' });
            console.log("Exported: " + files[i]);

        } catch (e) {
            console.log("Error with " + files[i] + ": " + e.message);
        }
    }
}
```

## File System with ObjC Bridge

```javascript
ObjC.import('Foundation');

function fileExists(posixPath) {
    var fm = $.NSFileManager.defaultManager;
    return fm.fileExistsAtPath($(posixPath).stringByStandardizingPath);
}

function getDesktopPath() {
    var paths = $.NSSearchPathForDirectoriesInDomains(
        $.NSDesktopDirectory,
        $.NSUserDomainMask,
        true
    );
    return ObjC.unwrap(paths.objectAtIndex(0));
}

// Usage
if (fileExists("/Users/me/deck.key")) {
    var doc = Keynote.open(Path("/Users/me/deck.key"));
}
```
