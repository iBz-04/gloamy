# Keynote Automation with PyXA (Python)

PyXA provides a Pythonic interface to control Apple Keynote on macOS via Apple's Scripting Bridge.

## Installation

```bash
pip install pyxa
```

## Bootstrapping

```python
import PyXA

# Launch or activate Keynote
keynote = PyXA.Application("Keynote")
keynote.activate()
```

## Presentation Management

### Creating Presentations

```python
# Create new presentation
pres = keynote.presentations.new()

# Create with specific theme
pres = keynote.presentations.new(theme="White")
```

### Opening Presentations

```python
# Open existing file
pres = keynote.open("/Users/me/Documents/Demo.key")
```

### Saving Presentations

```python
# Save to new location
pres.save("/Users/me/Desktop/Output.key")

# Save existing
pres.save()
```

## Slide Operations

### Creating Slides

```python
# Add slide at end
slide = pres.slides.new_after(pres.slides[-1])

# Add slide with specific master
blank_master = pres.master_slides["Blank"]
slide = pres.slides.new(base_slide=blank_master)
```

### Accessing Slides

```python
# By index
first_slide = pres.slides[0]

# Current slide
current = pres.current_slide

# Set current slide (navigation)
pres.current_slide = pres.slides[2]
```

## Content Manipulation

### Text Items

```python
# Add text box
text_box = slide.text_items.new()
text_box.text = "Automated Content"
text_box.position = (50, 50)
text_box.width = 300
text_box.height = 100

# Access title placeholder
title = slide.default_title_item
if title:
    title.text = "Slide Title"

# Access body placeholder
body = slide.default_body_item
if body:
    body.text = "Bullet point content"
```

### Shapes

```python
# Add rectangle
shape = slide.shapes.new_rectangle(
    position=(100, 100),
    size=(200, 100)
)
shape.text_body.text = "Shape Text"
shape.opacity = 80
shape.rotation = 15
```

### Images

```python
# Add image
image = slide.images.new(
    path="/Users/me/Assets/diagram.png",
    position=(400, 200)
)
image.width = 600  # Height auto-scales
```

## Tables

```python
# Create table
table = slide.tables.new(
    position=(100, 200),
    size=(800, 400),
    rows=4,
    columns=3
)

# Populate data
data = [
    ["Metric", "Q1", "Q2"],
    ["Revenue", "1.2M", "1.5M"],
    ["Cost", "500k", "450k"],
    ["Profit", "700k", "1.05M"]
]

for r, row in enumerate(data):
    for c, value in enumerate(row):
        table.rows[r].cells[c].value = value
```

## Formatting

### Text Styling

```python
# Font properties
text_box.font_size = 24
text_box.font_name = "Helvetica Neue"
text_box.font_color = PyXA.XAColor.white
text_box.bold = True
```

### Shape Styling

```python
# Fill and colors
shape.fill_color = PyXA.XAColor.blue
shape.stroke_color = PyXA.XAColor.black
shape.stroke_width = 2
```

### Themes

```python
# Change document theme
pres.document_theme = keynote.document_themes["White"]

# List available themes
for theme in keynote.document_themes:
    print(theme.name)
```

## Transitions and Animation

### Slide Transitions

```python
# Set transition
slide.transition_effect = "dissolve"
slide.transition_duration = 1.5
slide.automatic_transition = True
```

### Magic Move Pattern

```python
# Create base slide with shape
slide_a = pres.slides.new(base_slide=pres.master_slides["Blank"])
shape = slide_a.shapes.new_rectangle(position=(100, 500), size=(100, 100))
shape.text_body.text = "Start"

# Duplicate slide (objects maintain identity)
slide_b = slide_a.duplicate()

# Transform shape on destination
slide_b.shapes[0].position = (1500, 200)
slide_b.shapes[0].opacity = 50
slide_b.shapes[0].rotation = 45

# Apply Magic Move to origin slide
slide_a.transition_effect = "magic move"
slide_a.transition_duration = 2.0
```

## Export

### PDF Export

```python
pres.export(
    "/Users/me/Desktop/Presentation.pdf",
    export_format=PyXA.KY_ExportFormat.pdf
)
```

### PowerPoint Export

```python
pres.export(
    "/Users/me/Desktop/Deck.pptx",
    export_format=PyXA.KY_ExportFormat.powerpoint
)
```

### Video Export

```python
pres.export(
    "/Users/me/Desktop/Video.mov",
    export_format=PyXA.KY_ExportFormat.quicktime
)
```

## Complete Example

```python
import PyXA
from datetime import date

def generate_keynote_report(output_path, title, data):
    """Generate a Keynote presentation with PyXA."""
    keynote = PyXA.Application("Keynote")
    keynote.activate()

    # Create presentation with White theme
    pres = keynote.presentations.new(theme="White")

    # Title slide (first slide exists by default)
    title_slide = pres.slides[0]
    title_slide.default_title_item.text = title
    title_slide.default_body_item.text = f"Generated: {date.today()}"

    # Data slide
    blank = pres.master_slides["Blank"]
    data_slide = pres.slides.new(base_slide=blank)

    # Add table
    table = data_slide.tables.new(
        position=(100, 100),
        size=(800, 500),
        rows=len(data),
        columns=len(data[0])
    )

    # Populate table
    for r, row in enumerate(data):
        for c, value in enumerate(row):
            table.rows[r].cells[c].value = str(value)

    # Export to PDF
    pres.export(output_path, export_format=PyXA.KY_ExportFormat.pdf)
    pres.close()

    return {"success": True, "path": output_path}

# Usage
data = [
    ["Service", "Status", "Uptime"],
    ["API", "Green", "99.9%"],
    ["Database", "Green", "100%"],
    ["Auth", "Yellow", "99.5%"]
]
generate_keynote_report("/Users/me/Desktop/Report.pdf", "Daily Report", data)
```

## PyXA vs JXA Comparison

| Feature | PyXA | JXA |
|---------|------|-----|
| Create slide | `pres.slides.new()` | `Keynote.Slide({...}); doc.slides.push(slide)` |
| Set text | `item.text = "..."` | `item.objectText = "..."` |
| Transition | `slide.transition_effect = "..."` | `slide.transitionProperties = {...}` |
| Charts | Built-in support | Requires AppleScript bridge |

## Notes

- PyXA provides a more Pythonic API than JXA
- Both use Apple's Scripting Bridge under the hood
- Chart creation may still require special handling
- Master slide names vary by theme
- Test with `keynote.document_themes` to list available themes
