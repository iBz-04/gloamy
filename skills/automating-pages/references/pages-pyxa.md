# Pages Automation with PyXA (Python)

PyXA provides a Pythonic interface to control Apple Pages on macOS via Apple's Scripting Bridge.

## Installation

```bash
pip install pyxa
```

## Bootstrapping

```python
import PyXA

# Launch or activate Pages
pages = PyXA.Application("Pages")
pages.activate()
```

## Document Management

### Creating Documents

```python
# Create new document
doc = pages.documents.new()

# Create with specific template
doc = pages.documents.new(template="Blank")
```

### Opening Documents

```python
# Open existing file
doc = pages.open("/Users/me/Documents/Report.pages")
```

### Saving Documents

```python
# Save to new location
doc.save("/Users/me/Desktop/Output.pages")

# Save existing
doc.save()
```

## Content Manipulation

### Working with Body Text

```python
# Access document body
body = doc.body_text

# Insert text at end
body.insert("First paragraph\n", at=body.end())
body.insert("Second paragraph.\n", at=body.end())

# Insert at specific position
body.insert("Introduction\n\n", at=0)
```

### Working with Paragraphs

```python
# Access paragraphs
first_para = doc.paragraphs[0]
all_paras = doc.paragraphs

# Get text content
text = first_para.text

# Set paragraph text
first_para.text = "Updated content"
```

## Formatting

### Paragraph Styles

```python
# Apply predefined style
body.paragraphs[0].style_name = "Heading"
body.paragraphs[1].style_name = "Body"
body.paragraphs[2].style_name = "Title"
```

### Text Formatting

```python
# Font properties
body.paragraphs[0].font_size = 24
body.paragraphs[0].font_name = "Helvetica Neue"
body.paragraphs[0].bold = True

# Colors
body.paragraphs[0].font_color = PyXA.XAColor.blue

# Alignment
body.paragraphs[0].alignment = PyXA.PG_ParagraphAlignment.center
```

### Bulk Formatting

```python
# Format multiple paragraphs
body.paragraphs[0:3].bold = True
body.paragraphs[0:3].font_size = 14
```

## Images

```python
# Insert image
image = doc.images.new(
    path="/Users/me/Assets/diagram.png",
    bounds=(100, 100, 500, 300)  # x, y, width, height
)

# Adjust position
image.position = (200, 150)
image.width = 400  # Height auto-scales
```

## Tables

```python
# Create table
table = doc.tables.new(
    rows=4,
    columns=3
)

# Populate data
data = [
    ["Name", "Role", "Department"],
    ["Alice", "Engineer", "R&D"],
    ["Bob", "Designer", "UX"],
    ["Carol", "Manager", "Ops"]
]

for r, row in enumerate(data):
    for c, value in enumerate(row):
        table.rows[r].cells[c].value = value

# Format header row
table.rows[0].bold = True
table.rows[0].background_color = PyXA.XAColor.light_gray
```

## Export

### PDF Export

```python
doc.export(
    "/Users/me/Desktop/Document.pdf",
    export_format=PyXA.PG_ExportFormat.pdf
)
```

### Word Export

```python
doc.export(
    "/Users/me/Desktop/Document.docx",
    export_format=PyXA.PG_ExportFormat.word
)
```

### Plain Text Export

```python
doc.export(
    "/Users/me/Desktop/Document.txt",
    export_format=PyXA.PG_ExportFormat.plain_text
)
```

## Complete Example

```python
import PyXA
from datetime import date

def create_pages_document(output_path, title, sections):
    """Create a formatted Pages document with PyXA."""
    pages = PyXA.Application("Pages")
    pages.activate()

    # Create document
    doc = pages.documents.new()
    body = doc.body_text

    # Add title
    body.insert(f"{title}\n\n")
    doc.paragraphs[0].style_name = "Title"
    doc.paragraphs[0].alignment = PyXA.PG_ParagraphAlignment.center

    # Add date
    body.insert(f"Generated: {date.today()}\n\n", at=body.end())

    # Add sections
    for section_title, content in sections:
        # Section heading
        body.insert(f"{section_title}\n", at=body.end())
        para_idx = len(doc.paragraphs) - 1
        doc.paragraphs[para_idx].style_name = "Heading"

        # Section content
        body.insert(f"{content}\n\n", at=body.end())

    # Export to PDF
    doc.export(output_path, export_format=PyXA.PG_ExportFormat.pdf)
    doc.close()

    return {"success": True, "path": output_path}

# Usage
sections = [
    ("Overview", "This document provides a summary of Q4 results."),
    ("Key Findings", "Revenue increased 25% year over year."),
    ("Recommendations", "Continue current strategy with focus on growth.")
]
create_pages_document("/Users/me/Desktop/Report.pdf", "Quarterly Report", sections)
```

## Template-Based Document

```python
import PyXA

def fill_template(template_path, output_path, replacements):
    """Fill a Pages template with data."""
    pages = PyXA.Application("Pages")
    pages.activate()

    # Open template
    doc = pages.open(template_path)

    # Replace placeholders
    for placeholder, value in replacements.items():
        doc.body_text.replace(placeholder, value)

    # Save as new document
    doc.save(output_path)
    doc.close()

# Usage
replacements = {
    "{{NAME}}": "John Smith",
    "{{DATE}}": "January 14, 2026",
    "{{COMPANY}}": "Acme Corp"
}
fill_template(
    "/Users/me/Templates/Letter.pages",
    "/Users/me/Desktop/Letter_Filled.pages",
    replacements
)
```

## PyXA vs JXA Comparison

| Feature | PyXA | JXA |
|---------|------|-----|
| Insert text | `body.insert("text")` | Limited API |
| Styles | `para.style_name = "..."` | Limited support |
| Export | `doc.export(path, format)` | `doc.export({...})` |
| API Coverage | Higher-level | Lower-level |

## Notes

- Pages has more limited scripting support than Word
- Template-based approach often works better than programmatic creation
- PyXA provides a cleaner API than JXA for Pages
- Style names must match document's available styles
- Test available styles with `doc.character_styles` and `doc.paragraph_styles`
