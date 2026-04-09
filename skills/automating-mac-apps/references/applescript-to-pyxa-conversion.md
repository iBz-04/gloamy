# AppleScript to PyXA Conversion Guide

## Table of Contents
1. [Introduction](#1-introduction)
2. [Installation and Setup](#2-installation-and-setup)
3. [Basic Syntax Mapping](#3-basic-syntax-mapping)
4. [Object Model Differences](#4-object-model-differences)
5. [Error Handling Conversions](#5-error-handling-conversions)
6. [Application-Specific Conversions](#6-application-specific-conversions)
7. [Troubleshooting](#7-troubleshooting)
8. [Comprehensive Examples](#8-comprehensive-examples)
9. [Pattern Reference](#9-pattern-reference-10-examples)
10. [Migration Strategies](#10-migration-strategies)

## 1. Introduction

AppleScript has been macOS's primary automation language since 1993, but PyXA (Python for Apple Automation) offers modern Python-based alternatives. This guide focuses on migrating from legacy AppleScript workflows to contemporary PyXA approaches.

### What is PyXA?
PyXA is a Python wrapper around Apple's Scripting Bridge framework that enables AppleScript-like control over macOS applications using Python syntax.

### Why Migrate?
- Modern Python ecosystem
- Better error handling
- Cross-platform compatibility
- Rich Python libraries integration
- Improved maintainability

## 2. Installation and Setup

### Installing PyXA
```bash
pip install mac-pyxa
```

### Basic Import
```python
import PyXA
```

## 3. Basic Syntax Mapping

### Application References
**AppleScript:**
```applescript
tell application "Safari"
    -- commands
end tell
```

**PyXA:**
```python
app = PyXA.Application("Safari")
# commands
```

### Property Access
**AppleScript:**
```applescript
name of window 1
```

**PyXA:**
```python
app.windows()[0].name
```

### Method Calls
**AppleScript:**
```applescript
activate
```

**PyXA:**
```python
app.activate()
```

## 4. Object Model Differences

### Collections and Indexing
**AppleScript:** 1-based indexing
```applescript
item 1 of windows
```

**PyXA:** 0-based indexing  
```python
app.windows()[0]
```

### List Operations
**AppleScript:**
```applescript
set myList to {1, 2, 3}
set item 2 of myList to 5
```

**PyXA:**
```python
my_list = [1, 2, 3]
my_list[1] = 5
```

### Record/Dictionary Access
**AppleScript:**
```applescript
foo of {foo: "bar", spam: "eggs"}
```

**PyXA:**
```python
data = {"foo": "bar", "spam": "eggs"}
data["foo"]
```

## 5. Error Handling Conversions

### Try-Catch Blocks
**AppleScript:**
```applescript
try
    -- risky code
on error errMsg number errNum
    -- handle error
end try
```

**PyXA:**
```python
try:
    # risky code
except Exception as e:
    # handle error
```

### AppleScript-Specific Errors
**AppleScript:**
```applescript
tell application "Finder"
    if not (exists file "test.txt") then
        error "File not found"
    end if
end tell
```

**PyXA:**
```python
from PyXA import XAErrors

try:
    file_path = "/path/to/test.txt"
    if not os.path.exists(file_path):
        raise XAErrors.AppleScriptError("File not found")
except XAErrors.AppleScriptError as e:
    print(f"Error: {e}")
```

## 6. Application-Specific Conversions

### Finder Operations
**AppleScript:**
```applescript
tell application "Finder"
    set desktopFiles to files of desktop
end tell
```

**PyXA:**
```python
finder = PyXA.Application("Finder")
desktop_files = finder.desktop().files()
```

### System Events (UI Scripting)
**AppleScript:**
```applescript
tell application "System Events"
    tell process "Safari"
        click button "Allow" of window 1
    end tell
end tell
```

**PyXA:**
```python
system_events = PyXA.Application("System Events")
safari_process = system_events.processes().by_name("Safari")
safari_process.windows()[0].buttons().by_name("Allow").click()
```

## 7. Troubleshooting

### Common Issues

1. **IndexError on Collections**
   - **Problem:** `app.windows()[0]` fails when no windows exist
   - **Solution:** Check collection length first
   ```python
   windows = app.windows()
   if len(windows) > 0:
       front_window = windows[0]
   ```

2. **Application Not Scriptable**
   - **Problem:** Some apps don't support scripting
   - **Solution:** Use UI scripting or AppleScript bridge
   ```python
   script = PyXA.AppleScript('tell application "App" to activate')
   script.run()
   ```

3. **Permission Issues**
   - **Problem:** macOS security restrictions
   - **Solution:** Grant accessibility permissions in System Settings

4. **Type Conversion Issues**
   - **Problem:** PyXA returns different types than expected
   - **Solution:** Check object types and convert explicitly
   ```python
   result = app.some_property
   if isinstance(result, PyXA.XATypes.XAText):
       text_value = str(result)
   ```

5. **Method Not Found**
   - **Problem:** PyXA method names differ from AppleScript
   - **Solution:** Check PyXA documentation or use `dir()` to explore available methods

### Debugging Tips
- Use `print(type(obj))` to check object types
- Use `dir(obj)` to see available methods/properties
- Enable verbose error messages
- Test AppleScript portions separately first

## 8. Comprehensive Examples

### Example 1: Safari Tab Management

**Original AppleScript:**
```applescript
tell application "Safari"
    set tabList to {}
    repeat with t in tabs of window 1
        set end of tabList to (name of t & " - " & URL of t)
    end repeat
    return tabList
end tell
```

**PyXA Conversion:**
```python
import PyXA

def get_safari_tabs():
    """Get list of Safari tab titles and URLs"""
    try:
        safari = PyXA.Application("Safari")
        tabs = safari.windows()[0].tabs()
        
        tab_list = []
        for tab in tabs:
            tab_info = f"{tab.name} - {tab.url}"
            tab_list.append(tab_info)
        
        return tab_list
    except IndexError:
        return ["No Safari windows open"]
    except Exception as e:
        return [f"Error: {str(e)}"]

# Usage
tabs = get_safari_tabs()
for tab in tabs:
    print(tab)
```

### Example 2: File Organization Script

**Original AppleScript:**
```applescript
tell application "Finder"
    set sourceFolder to folder "Downloads" of home
    set destFolder to folder "Documents" of home
    
    set fileList to every file of sourceFolder whose name extension is "pdf"
    
    repeat with aFile in fileList
        move aFile to destFolder
    end repeat
end tell
```

**PyXA Conversion:**
```python
import PyXA
import os

def organize_pdf_files():
    """Move PDF files from Downloads to Documents folder"""
    try:
        finder = PyXA.Application("Finder")
        
        # Get source and destination folders
        home_path = os.path.expanduser("~")
        downloads_path = os.path.join(home_path, "Downloads")
        documents_path = os.path.join(home_path, "Documents")
        
        source_folder = finder.folders().by_path(downloads_path)
        dest_folder = finder.folders().by_path(documents_path)
        
        if not source_folder or not dest_folder:
            raise ValueError("Source or destination folder not found")
        
        # Find PDF files
        pdf_files = []
        for item in source_folder.items():
            if hasattr(item, 'name_extension') and item.name_extension == "pdf":
                pdf_files.append(item)
        
        # Move files
        moved_count = 0
        for pdf_file in pdf_files:
            pdf_file.move_to(dest_folder)
            moved_count += 1
        
        return f"Successfully moved {moved_count} PDF files"
    
    except Exception as e:
        return f"Error organizing files: {str(e)}"

# Usage
result = organize_pdf_files()
print(result)
```

## 9. Pattern Reference (10 Examples)

### 1. Application Activation
**AppleScript:**
```applescript
tell application "TextEdit" to activate
```
**PyXA:**
```python
PyXA.Application("TextEdit").activate()
```

### 2. Getting Window Count
**AppleScript:**
```applescript
tell application "Safari"
    count of windows
end tell
```
**PyXA:**
```python
len(PyXA.Application("Safari").windows())
```

### 3. Creating New Document
**AppleScript:**
```applescript
tell application "TextEdit"
    make new document
end tell
```
**PyXA:**
```python
PyXA.Application("TextEdit").new_document()
```

### 4. Getting Selected Text
**AppleScript:**
```applescript
tell application "TextEdit"
    selected text of front document
end tell
```
**PyXA:**
```python
app = PyXA.Application("TextEdit")
app.front_document.selected_text
```

### 5. File Existence Check
**AppleScript:**
```applescript
tell application "Finder"
    exists file "test.txt" of home
end tell
```
**PyXA:**
```python
import os
os.path.exists(os.path.expanduser("~/test.txt"))
```

### 6. Folder Contents
**AppleScript:**
```applescript
tell application "Finder"
    every file of folder "Desktop" of home
end tell
```
**PyXA:**
```python
finder = PyXA.Application("Finder")
desktop = finder.desktop()
desktop.items()
```

### 7. System Beep
**AppleScript:**
```applescript
beep
```
**PyXA:**
```python
import subprocess
subprocess.run(["afplay", "/System/Library/Sounds/Basso.aiff"])
```

### 8. Current Date/Time
**AppleScript:**
```applescript
current date
```
**PyXA:**
```python
import datetime
datetime.datetime.now()
```

### 9. String Concatenation
**AppleScript:**
```applescript
"Hello " & "World"
```
**PyXA:**
```python
"Hello " + "World"
```

### 10. List Operations
**AppleScript:**
```applescript
set myList to {1, 2, 3}
set end of myList to 4
```
**PyXA:**
```python
my_list = [1, 2, 3]
my_list.append(4)
```

## 10. Migration Strategies

### Gradual Migration Approach
1. **Identify Core Functionality:** Start with simple, self-contained AppleScript functions
2. **Create PyXA Equivalents:** Convert one function at a time
3. **Test Thoroughly:** Ensure behavior matches exactly
4. **Integrate Python Libraries:** Leverage Python's ecosystem for enhanced functionality
5. **Refactor for Maintainability:** Use Python best practices

### Key Benefits of Migration
- **Type Safety:** Python's type hints and modern IDE support
- **Testing:** Unit testing frameworks (pytest, unittest)
- **Version Control:** Better integration with git and modern development workflows
- **Package Management:** pip and requirements.txt for dependency management
- **Cross-Platform Potential:** Easier to adapt for non-macOS environments

### Best Practices
- Keep original AppleScript as reference during conversion
- Add comprehensive error handling
- Use Python's logging instead of AppleScript's `log` command
- Leverage Python's data structures over AppleScript's limited types
- Consider async/await for long-running operations

This guide provides a foundation for migrating AppleScript automations to PyXA. The PyXA documentation at https://skaplanofficial.github.io/PyXA/ contains comprehensive API references for specific applications.