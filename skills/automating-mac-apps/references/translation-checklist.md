# AppleScript → JXA Translation Checklist

Use this after prototyping in AppleScript. This guide covers systematic translation patterns for converting AppleScript automation to JavaScript for Automation (JXA).

## Translation Tools & Resources

### Modern Python Alternatives (Recommended)
**PyXA** - Modern Python replacement for JXA:
- **Advantages**: Active development, better documentation, modern syntax
- **Installation**: `pip install mac-pyxa` (latest: 0.3.0.1)
- **Requirements**: Python 3.10+, PyObjC 9.x
- **Documentation**: https://skaplanofficial.github.io/PyXA/
- **Migration**: Often easier to rewrite JXA in PyXA than translate directly

**PyObjC** - Direct Python-Objective-C bridge:
- **Advantages**: Access to all macOS frameworks, execute AppleScript/JXA from Python
- **Installation**: `pip install pyobjc`
- **Documentation**: https://pyobjc.readthedocs.io

### Legacy JXA Resources
### Official Apple Resources
- [JavaScript for Automation Release Notes](https://developer.apple.com/library/archive/releasenotes/InterapplicationCommunication/RN-JavaScriptForAutomation/Articles/Introduction.html)
- [AppleScript Language Guide](https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/introduction/ASLR_intro.html)

### Community Tools
- **osascript2js**: Command-line tool for basic AppleScript → JXA conversion
- **JXA-Cookbook**: GitHub repository with translation examples
- **Script Debugger**: Commercial tool with built-in AppleScript ↔ JXA conversion

### Online Converters
- Various GitHub Gist tools for syntax translation
- Community forums (MacScripter, Reddit r/applescript) with conversion examples

## Core Syntax Mapping

### Application Context
| AppleScript | JXA | Notes |
|-------------|-----|-------|
| `tell application "App"` | `const app = Application("App");` | Use Application() constructor |
| `tell app "Finder"` | `const finder = Application("Finder");` | Standard app reference |
| `tell current application` | `const currentApp = Application.currentApplication();` | Self-reference |

### Property Access
| AppleScript | JXA | Notes |
|-------------|-----|-------|
| `name of document` | `document.name()` | Property reads are method calls |
| `set name of doc to "X"` | `doc.name = "X"` | Property writes are assignments |
| `get properties of doc` | `doc.properties()` | Properties method for all attributes |
| `exists document "test"` | `app.documents.byName("test") !== null` | Existence checking |

### Collections & Indexing
| AppleScript | JXA | Notes |
|-------------|-----|-------|
| `every document` | `app.documents()` | Returns array of objects |
| `document 1` | `app.documents[0]` | Zero-based indexing |
| `first document` | `app.documents[0]` | Same as indexing |
| `last document` | `app.documents[app.documents.length - 1]` | End of array |
| `front window` | `app.windows[0]` | Active/front window |

### Control Flow
| AppleScript | JXA | Notes |
|-------------|-----|-------|
| `if condition then` | `if (condition) {` | Standard JS syntax |
| `repeat with i from 1 to 10` | `for (let i = 0; i < 10; i++)` | Zero-based loops |
| `repeat while condition` | `while (condition)` | Standard while loop |
| `try ... on error` | `try { ... } catch (error)` | Exception handling |

### Common Patterns
| AppleScript | JXA | Notes |
|-------------|-----|-------|
| `whose` filters | `filter()` or `whose()` | JXA supports both |
| `tell System Events` | `Application("System Events")` | UI scripting context |
| `delay 1` | `delay(1)` | Delay in seconds |
| `do shell script "cmd"` | `app.doShellScript("cmd")` | Shell execution |

## Translation Verification Steps

### Step-by-Step Conversion Process
1. **Test AppleScript Original**: Run AppleScript in Script Editor to confirm it works
2. **Identify Application Context**: Map `tell application` blocks to `Application()` calls
3. **Convert Property Access**: Change `property of object` to `object.property()` or `object.property = value`
4. **Handle Collections**: Convert `every item` and indexing to array operations
5. **Translate Control Flow**: Convert AppleScript conditionals and loops to JavaScript equivalents
6. **Handle Special Cases**: Convert `whose` filters, UI scripting, and shell commands
7. **Test JXA Version**: Run converted script and compare outputs
8. **Debug Issues**: Use `console.log()` for debugging, check for zero-based indexing errors

### Dictionary Verification
- **Script Editor Check**: File → Open Dictionary → Select target app → Verify command availability
- **Property Types**: Note whether properties return values or functions in JXA
- **Method Signatures**: Check if methods require parameters (e.g., `save({in: path})`)
- **UI Scripting**: Confirm accessibility permissions for System Events operations

### Whose Clause Translation
```javascript
// AppleScript
every document whose name contains "report"

// JXA Option 1: Native whose (may be slow)
const docs = app.documents.whose({name: {_contains: "report"}})();

// JXA Option 2: JavaScript filter (often faster)
const docs = app.documents().filter(doc => doc.name().includes("report"));
```

### Output Handling
- **CLI Integration**: Use `JSON.stringify(result)` for structured output
- **Debug Logging**: `console.log()` goes to stderr; use only for debugging
- **Error Handling**: Wrap in try/catch blocks for robust error reporting

## Common Translation Pitfalls & Solutions

### Core Language Differences

#### Function vs Property Access
**Pitfall**: JXA collections are often functions, not direct properties
```javascript
// Wrong - treats as property
const docs = app.documents; // Returns function reference

// Correct - call as function
const docs = app.documents(); // Returns array
```

#### Method Calls for Property Reads
**Pitfall**: Property reads require method calls, writes are direct assignments
```javascript
// AppleScript
set name of doc to "New Name"
get name of doc

// JXA
doc.name = "New Name";     // Assignment
const name = doc.name();   // Method call
```

### Error-Prone Patterns

#### Zero-Based Indexing
**Pitfall**: JXA uses zero-based indexing, AppleScript uses one-based
```javascript
// AppleScript
document 1      // First document

// JXA
app.documents[0]  // First document (zero-based)
```

#### Collection Length Checks
**Pitfall**: Accessing indexed elements without checking array length
```javascript
// Unsafe - crashes if no windows
const frontWindow = app.windows[0];

// Safe - check length first
const frontWindow = app.windows.length > 0 ? app.windows[0] : null;
```

### UI Scripting Considerations

#### System Events Requirements
**Pitfall**: UI scripting requires System Events application and accessibility permissions
```javascript
// AppleScript
tell application "System Events"
    click button 1 of window 1 of process "App"
end tell

// JXA
const systemEvents = Application("System Events");
systemEvents.processes.byName("App").windows[0].buttons[0].click();
```

#### Permission Setup
- Enable Accessibility permissions in System Settings → Privacy & Security
- Test UI scripting with simple operations first
- Handle permission denied errors gracefully

### Performance & Optimization

#### Whose vs Filter Performance
**Pitfall**: `whose` clauses can be slow on large collections
```javascript
// Slower - native whose
const largeSet = app.documents.whose({modifiedDate: {_greaterThan: date}})();

// Faster - JavaScript filter (for large datasets)
const largeSet = app.documents().filter(doc =>
    doc.modifiedDate() > date
);
```

#### Batch Operations
**Pitfall**: Individual operations in loops are inefficient
```javascript
// Inefficient - individual saves
docs.forEach(doc => doc.save());

// Better - batch where possible, or accept individual operations
docs.forEach(doc => {
    // Perform modifications
    doc.save(); // Individual saves may be necessary
});
```

### File System Operations

#### Path Object Requirements
**Pitfall**: Some apps require Path() objects, not strings
```javascript
// Wrong - string path
doc.save({in: "/path/to/file.txt"});

// Correct - Path object
doc.save({in: Path("/path/to/file.txt")});
```

#### File Existence Checks
**Pitfall**: JXA file operations don't auto-create directories
```javascript
// Check directory exists before saving
const fileManager = Application("Finder");
const parentDir = Path("/path/to").toString();

// Note: File existence checking varies by app dictionary
```

### Timing & Synchronization

#### Delay Units
**Pitfall**: `delay()` uses seconds, not milliseconds
```javascript
// AppleScript
delay 0.5  // Half second

// JXA
delay(0.5);  // Half second (same as AppleScript)
```

#### UI Operation Timing
**Pitfall**: UI operations need delays for proper execution
```javascript
systemEvents.processes.byName("App").windows[0].buttons[0].click();
delay(0.5); // Allow UI to respond
```

### Output & Integration

#### JSON Output for CLI
**Pitfall**: Mixing stdout/stderr in CLI tools
```javascript
// Correct - structured JSON output to stdout
console.log(JSON.stringify({result: data, count: data.length}));

// Debug only - stderr
console.error("Debug info:", debugData);
```

#### Error Handling
**Pitfall**: Unhandled exceptions break automation
```javascript
try {
    // Automation code
    const result = app.doSomething();
    console.log(JSON.stringify(result));
} catch (error) {
    console.error(`Error: ${error.message}`);
    process.exit(1); // Non-zero exit for CLI tools
}
```

### Cross-Version Compatibility

#### Dictionary Changes
**Pitfall**: App dictionaries change between macOS versions
- Always test on target macOS version
- Check Script Editor dictionary for current method signatures
- Handle missing methods gracefully with try/catch

#### Deprecation Warnings
**Note**: JXA and AppleScript are legacy technologies. Consider modern alternatives like Shortcuts for new automation projects.

## Complete Translation Examples

### Example 1: Basic Document Operations

**AppleScript Original:**
```applescript
tell application "TextEdit"
    make new document
    set text of front document to "Hello World"
    save front document in file "output.txt"
end tell
```

**JXA Translation:**
```javascript
const textEdit = Application("TextEdit");

// Create new document
textEdit.documents.push(textEdit.Document());

// Set content and save
const doc = textEdit.documents[0];
doc.text = "Hello World";
doc.save({in: Path("output.txt")});
```

### Example 2: Filtered Collection Processing

**AppleScript Original:**
```applescript
tell application "Finder"
    set targetFolder to folder "Documents" of home
    set largeFiles to every file of targetFolder whose size > 1000000
    repeat with aFile in largeFiles
        -- Process large files
        log name of aFile
    end repeat
end tell
```

**JXA Translation:**
```javascript
const finder = Application("Finder");
const homeFolder = Path.home();
const targetFolder = `${homeFolder}/Documents`;

// Get files larger than 1MB
const largeFiles = finder.folders.byName(targetFolder)
    .items.whose({_and: [
        {kind: "file"},
        {size: {_greaterThan: 1000000}}
    ]})();

// Process each file
largeFiles.forEach(file => {
    console.log(file.name());
    // Additional processing here
});
```

### Example 3: UI Scripting with Error Handling

**AppleScript Original:**
```applescript
tell application "System Events"
    tell process "Safari"
        click button "New Tab" of window 1
        delay 0.5
        set value of text field 1 of window 1 to "https://example.com"
        keystroke return
    end tell
end tell
```

**JXA Translation:**
```javascript
try {
    const systemEvents = Application("System Events");
    const safari = systemEvents.processes.byName("Safari");

    // Click New Tab button
    safari.windows[0].buttons.byName("New Tab").click();

    // Wait for UI to update
    delay(0.5);

    // Enter URL and submit
    const addressField = safari.windows[0].textFields[0];
    addressField.value = "https://example.com";

    // Simulate Return key
    systemEvents.keystroke("\r");

} catch (error) {
    console.error(`UI automation failed: ${error.message}`);
    // Handle accessibility permission issues
}
```

**PyXA Translation (Recommended Modern Approach):**
```python
import PyXA

try:
    safari = PyXA.Safari()

    # Open new tab and navigate
    safari.open_location("https://example.com")

    # PyXA handles UI interactions more reliably than JXA
    # No need for explicit UI scripting in many cases

except Exception as e:
    print(f"Safari automation failed: {e}")
```

### Example 4: Calendar Event Creation

**JXA Original:**
```javascript
const Calendar = Application("Calendar");
const event = Calendar.Event({
    summary: "Meeting",
    startDate: new Date(),
    endDate: new Date(Date.now() + 3600000)  // 1 hour later
});
Calendar.events.push(event);
```

**PyXA Translation:**
```python
import PyXA

calendar = PyXA.Calendar()

# Create event
event = calendar.events().push({
    "summary": "Meeting",
    "start_date": datetime.now(),
    "end_date": datetime.now() + timedelta(hours=1)
})
```

## Testing & Validation

### Unit Testing Approach
1. **Isolate Components**: Test each translation segment separately
2. **Compare Outputs**: Ensure JXA and AppleScript produce identical results
3. **Handle Edge Cases**: Test with empty collections, missing files, permission errors
4. **Performance Comparison**: Time operations and optimize bottlenecks

### Common Testing Scenarios
- Empty collections (`documents()` returns `[]`)
- Missing applications (app not installed)
- Permission denied (accessibility/UI scripting)
- File system errors (path not found, permission denied)
- Network timeouts (for remote operations)

### Debugging Tools
- **Script Editor**: Test JXA directly with debugging console
- **Console.app**: Check system logs for automation errors
- **Activity Monitor**: Monitor process behavior during automation
- **Accessibility Inspector**: Debug UI element hierarchies (Xcode tool)
