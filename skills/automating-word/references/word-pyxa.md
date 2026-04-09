# Word Automation with PyXA (Python)

PyXA is a Python library for macOS automation that wraps Apple's Scripting Bridge. It provides a Pythonic interface to control Microsoft Word.

## Installation

```bash
pip install pyxa
```

## Bootstrapping

```python
import PyXA

# Launch or activate Word
word = PyXA.Application("Microsoft Word")
word.activate()  # Launches if not running
```

## Document Lifecycle

### Creating Documents

```python
# Create new document
doc = word.documents.new()
```

### Opening Documents

```python
# Open existing document
doc = word.open("/Users/me/Documents/Report.docx")
```

### Saving Documents

```python
# Save to new location
doc.save_as("/Users/me/Desktop/Output.docx")

# Save existing
doc.save()
```

## Content Manipulation

### Inserting Text

```python
# Insert text at end
doc.text_objects.insert("Hello, PyXA!", at=doc.text_objects.end())

# Insert at specific position
doc.content.insert("Introduction\n", at=0)
```

### Working with Paragraphs

```python
# Access paragraphs
first_para = doc.paragraphs[0]
all_paras = doc.paragraphs

# Get paragraph text
text = first_para.text
```

## Formatting

### Paragraph Formatting

```python
# Bold, font size, alignment
doc.paragraphs[0].bold = True
doc.paragraphs[0].font_size = 14
doc.paragraphs[0].alignment = PyXA.WD_ParagraphAlignment.center

# Bulk formatting
doc.paragraphs[0:5].bold = True
```

### Character Formatting

```python
# Font properties
doc.characters.font_name = "Arial"
doc.characters.font_size = 12
doc.characters.font_color = PyXA.XAColor.blue
```

## Tables

```python
# Add table at end of document
table = doc.tables.add(
    range_object=doc.range(start=doc.characters.end() - 1),
    rows=3,
    columns=4
)

# Populate cells
table.cells.item(1, 1).text = "Header 1"
table.cells.item(1, 2).text = "Header 2"
table.cells.item(2, 1).text = "Data A"
table.cells.item(2, 2).text = "Data B"
```

## Find and Replace

```python
# Simple find/replace
doc.content.find("old text").replace("new text")

# Replace all occurrences
doc.content.replace_all("foo", "bar")
```

## Export to PDF

```python
# Export as PDF
doc.save_as("/Users/me/Desktop/Report.pdf",
            file_format=PyXA.WD_FileFormat.pdf)
```

## Complete Example

```python
import PyXA

def create_report(output_path, title, sections):
    """Generate a formatted Word document with PyXA."""
    word = PyXA.Application("Microsoft Word")
    word.activate()

    # Create document
    doc = word.documents.new()

    # Add title
    doc.content.insert(f"{title}\n\n")
    doc.paragraphs[0].bold = True
    doc.paragraphs[0].font_size = 24
    doc.paragraphs[0].alignment = PyXA.WD_ParagraphAlignment.center

    # Add sections
    for section_title, content in sections:
        doc.content.insert(f"\n{section_title}\n", at=doc.content.end())
        doc.content.insert(f"{content}\n", at=doc.content.end())

    # Save as PDF
    doc.save_as(output_path, file_format=PyXA.WD_FileFormat.pdf)
    doc.close()

    return {"success": True, "path": output_path}

# Usage
sections = [
    ("Overview", "This report covers Q4 performance."),
    ("Results", "Revenue increased by 25%."),
    ("Conclusion", "Strong quarter overall.")
]
create_report("/Users/me/Desktop/Q4Report.pdf", "Q4 Report", sections)
```

## PyXA vs JXA Comparison

| Feature | PyXA (Python) | JXA (JavaScript) |
|---------|---------------|------------------|
| Syntax | Pythonic | JavaScript |
| Installation | `pip install pyxa` | Built-in |
| Data Structures | Python lists/dicts | JS arrays/objects |
| Error Handling | try/except | try/catch |
| Performance | Similar (both use Scripting Bridge) | Similar |
| IDE Support | Full Python tooling | Script Editor only |

## Notes

- PyXA wraps Apple's Scripting Bridge (same foundation as JXA)
- Install via `pip install pyxa` (requires macOS)
- Apps must support AppleScript/Scripting Bridge
- Property assignment is direct: `obj.property = value`
- Use enums for format constants (e.g., `PyXA.WD_FileFormat.pdf`)
