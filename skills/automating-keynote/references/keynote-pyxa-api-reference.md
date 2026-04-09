# PyXA Keynote Module API Reference

> **New in PyXA version 0.0.2** - Control macOS Keynote using JXA-like syntax from Python.

This reference documents all classes, methods, properties, and enums in the PyXA Keynote module. For practical examples and usage patterns, see [keynote-pyxa.md](keynote-pyxa.md).

## Contents

- [Class Hierarchy](#class-hierarchy)
- [XAKeynoteApplication](#xakeynoteapplication)
- [XAKeynoteDocument](#xakeynotedocument)
- [XAKeynoteSlide](#xakeynoteslide)
- [XAKeynoteSlideLayout](#xakeynoteslidelayout)
- [XAKeynoteTheme](#xakeynotetheme)
- [XAKeynoteTransitionSettings](#xakeynotetransitionsettings)
- [XAKeynoteWindow](#xakeynotewindow)
- [XAKeynoteContainer](#xakeynotecontainer)
- [List Classes](#list-classes)
- [Enumerations](#enumerations)
- [Quick Reference Tables](#quick-reference-tables)

---

## Class Hierarchy

```
XAObject
├── XAKeynoteApplication (XAiWorkApplication)
│   ├── XAKeynoteDocument (XAiWorkDocument)
│   │   ├── XAKeynoteSlide (XAKeynoteContainer)
│   │   │   └── XAKeynoteSlideLayout
│   │   └── XAKeynoteTheme
│   └── XAKeynoteWindow (XAiWorkWindow)
└── XAKeynoteTransitionSettings
```

---

## XAKeynoteApplication

**Bases:** `XAiWorkApplication`

Main entry point for interacting with Keynote.app.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `playing` | `bool` | Whether a slideshow is currently playing |
| `slide_switcher_visible` | `bool` | Whether the slide switcher is visible |
| `properties` | `dict` | All properties of the Keynote application |

### Methods

#### `documents(filter=None) → XAKeynoteDocumentList`

Returns a list of open documents matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
import PyXA
app = PyXA.Application("Keynote")
docs = app.documents()
for doc in docs:
    print(doc.name)
```

#### `themes(filter=None) → XAKeynoteThemeList`

Returns available themes matching the filter.

**Parameters:**
- `filter` (`dict | None`) - Property-value pairs to filter by

**Example:**
```python
themes = app.themes()
print(themes.name())
# ['Basic White', 'Basic Black', 'Classic White', ...]
```

#### `new_document(file_path='./Untitled.key', theme=None) → XAKeynoteDocument`

Creates a new document.

**Parameters:**
- `file_path` (`str | XAPath`) - Path to create document at
- `theme` (`XAKeynoteTheme | None`) - Theme to initialize with

**Returns:** The newly created document

#### `make(specifier, properties=None, data=None)`

Creates a new element without adding to any list. Use `XAList.push()` to add.

**Parameters:**
- `specifier` (`str | ObjectType`) - Class name to create
- `properties` (`dict`) - Properties for the object
- `data` (`Any`) - Initialization data

#### `show_next() → XAKeynoteApplication`

Advance one slide or animation build during slideshow.

#### `show_previous() → XAKeynoteApplication`

Go back one slide or animation build during slideshow.

---

## XAKeynoteDocument

**Bases:** `XAiWorkDocument`

Represents an open Keynote presentation.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Document name |
| `id` | `str` | Unique identifier |
| `file` | `str` | Location on disk (if saved) |
| `modified` | `bool` | Whether modified since last save |
| `password_protected` | `bool` | Whether password protected |
| `document_theme` | `XAKeynoteTheme` | Assigned theme |
| `current_slide` | `XAKeynoteSlide` | Currently selected slide |
| `selection` | `XAiWorkiWorkItemList` | Currently selected items |
| `width` | `int` | Width in points (standard: 1080, wide: 1920) |
| `height` | `int` | Height in points (standard: 768, wide: 1080) |
| `auto_loop` | `bool` | Whether slideshow repeats |
| `auto_play` | `bool` | Whether auto-plays on open |
| `auto_restart` | `bool` | Whether restarts on idle |
| `maximum_idle_duration` | `int` | Seconds before restart on idle |
| `slide_numbers_showing` | `bool` | Whether slide numbers displayed |
| `properties` | `dict` | All document properties |

### Methods

#### `slides(filter=None) → XAKeynoteSlideList`

Returns slides matching the filter.

**Example:**
```python
doc = app.documents()[0]
all_slides = doc.slides()
filtered = doc.slides().greater_than("slideNumber", 5)
```

#### `slide_layouts(filter=None) → XAKeynoteSlideLayoutList`

Returns slide layouts (master slides) matching the filter.

#### `new_slide(properties) → XAKeynoteSlide`

Creates a new slide at the end of the presentation.

**Parameters:**
- `properties` (`dict`) - Properties for the new slide

#### `make_image_slides(files, set_titles=False, slide_layout=None) → XAKeynoteDocument`

Creates slides from image files.

**Parameters:**
- `files` (`list[str | XAPath]`) - Paths to image files
- `set_titles` (`bool`) - Set slide titles to filenames
- `slide_layout` (`XAKeynoteSlideLayout | None`) - Layout to use

#### `export(file_path=None, format=ExportFormat.PDF)`

Exports the slideshow.

**Parameters:**
- `file_path` (`str | XAPath | None`) - Export destination
- `format` (`ExportFormat`) - Export format (default: PDF)

**Example:**
```python
doc.export("/path/to/output.pdf", XAKeynoteApplication.ExportFormat.PDF)
doc.export("/path/to/output.pptx", XAKeynoteApplication.ExportFormat.MICROSOFT_POWERPOINT)
```

#### `save()`

Saves the document.

#### `start_from(slide) → XAKeynoteSlide`

Starts slideshow from the specified slide.

#### `stop()`

Stops the currently playing slideshow.

#### Slide Switcher Methods

| Method | Description |
|--------|-------------|
| `show_slide_switcher()` | Show the slide switcher |
| `hide_slide_switcher()` | Hide the slide switcher |
| `accept_slide_switcher()` | Advance to selected slide |
| `cancel_slide_switcher()` | Dismiss the switcher |
| `move_slide_switcher_forward()` | Advance one slide |
| `move_slide_switcher_backward()` | Go back one slide |

---

## XAKeynoteSlide

**Bases:** `XAKeynoteContainer`

Represents a single slide in a presentation.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `slide_number` | `int` | Index in the document |
| `base_layout` | `XAKeynoteSlideLayout` | Layout this slide is based on |
| `default_title_item` | `XAiWorkShape` | Default title container |
| `default_body_item` | `XAiWorkShape` | Default body container |
| `presenter_notes` | `XAText` | Presenter notes |
| `title_showing` | `bool` | Whether title is displayed |
| `body_showing` | `bool` | Whether body is displayed |
| `skipped` | `bool` | Whether slide is skipped |
| `transition_properties` | `dict` | Transition settings |
| `properties` | `dict` | All slide properties |

### Methods

#### `add_image(file_path) → XAiWorkImage`

Adds an image to the slide.

**Parameters:**
- `file_path` (`str | XAPath | XAURL`) - Path to image file

**Returns:** The newly created image object

#### `add_chart(row_names, column_names, data, type=..., group_by=...) → XAiWorkChart`

Adds a chart to the slide.

**Parameters:**
- `row_names` (`list[str]`) - Row labels
- `column_names` (`list[str]`) - Column labels
- `data` (`list[list[Any]]`) - 2D data array
- `type` (`int`) - Chart type (default: Line 2D)
- `group_by` (`int`) - Grouping schema

#### `delete()`

Deletes the slide.

#### `duplicate(location=None, position=-1) → XAKeynoteSlide`

Duplicates the slide.

**Parameters:**
- `location` (`XAKeynoteDocument | XAKeynoteSlideList | None`) - Target location
- `position` (`int`) - Index in target list (-1 for end)

**Example:**
```python
doc1 = app.documents()[0]
doc2 = app.documents()[1]
doc1.slides()[0].duplicate(doc2)  # Copy to another document
```

#### `move(location, position=-1) → XAKeynoteSlide`

Moves the slide to a new location.

**Parameters:**
- `location` (`XAKeynoteDocument | XAKeynoteSlideList`) - Target location
- `position` (`int`) - Index in target list (-1 for end)

---

## XAKeynoteSlideLayout

**Bases:** `XAKeynoteSlide`

Represents a slide layout (master slide).

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `name` | `str` | Name of the slide layout |

---

## XAKeynoteTheme

**Bases:** `XAObject`

Represents a Keynote theme.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | `str` | Unique identifier |
| `name` | `str` | Theme name |
| `properties` | `dict` | All theme properties |

---

## XAKeynoteTransitionSettings

**Bases:** `XAObject`

Properties common to all transitions.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `automatic_transition` | `float` | Auto-advance setting |
| `transition_delay` | `float` | Delay before transition |
| `transition_duration` | `float` | Transition duration |
| `transition_effect` | `Transition` | Transition type |

---

## XAKeynoteWindow

**Bases:** `XAiWorkWindow`

Represents a Keynote window.

### Properties

| Property | Type | Description |
|----------|------|-------------|
| `document` | `XAKeynoteDocument` | Document displayed in window |

---

## XAKeynoteContainer

**Bases:** `XAiWorkContainer`

Base class for containers (slides, layouts) in Keynote.

---

## List Classes

PyXA provides list wrapper classes with fast enumeration and bulk property access.

### XAKeynoteDocumentList

Bulk methods for document lists:

```python
docs = app.documents()
docs.auto_loop()          # → list[bool]
docs.auto_play()          # → list[bool]
docs.current_slide()      # → XAKeynoteSlideList
docs.document_theme()     # → XAKeynoteThemeList
docs.height()             # → list[int]
docs.width()              # → list[int]
docs.maximum_idle_duration()  # → list[int]
docs.slide_numbers_showing()  # → list[bool]
docs.properties()         # → list[dict]
```

Filter methods:
```python
docs.by_auto_loop(True)
docs.by_height(768)
docs.by_document_theme(theme)
```

### XAKeynoteSlideList

Bulk methods for slide lists:

```python
slides = doc.slides()
slides.slide_number()        # → list[int]
slides.base_layout()         # → XAKeynoteSlideLayoutList
slides.body_showing()        # → list[bool]
slides.title_showing()       # → list[bool]
slides.skipped()             # → list[bool]
slides.presenter_notes()     # → XATextList
slides.default_title_item()  # → XAiWorkShapeList
slides.default_body_item()   # → XAiWorkShapeList
slides.transition_properties()  # → list[dict]
```

Filter methods:
```python
slides.by_slide_number(5)
slides.by_skipped(False)
slides.by_base_layout(layout)
```

### XAKeynoteSlideLayoutList

```python
layouts = doc.slide_layouts()
layouts.name()           # → list[str]
layouts.by_name("Blank")
```

### XAKeynoteThemeList

```python
themes = app.themes()
themes.id()      # → list[str]
themes.name()    # → list[str]
themes.by_name("Basic White")
themes.by_id("theme-id")
```

---

## Enumerations

### ExportFormat

Export format options.

| Value | Description |
|-------|-------------|
| `PDF` | PDF document |
| `JPEG` | JPEG images |
| `PNG` | PNG images |
| `TIFF` | TIFF images |
| `HTML` | HTML web export |
| `KEYNOTE` | Native Keynote format |
| `KEYNOTE_09` | Keynote '09 format |
| `MICROSOFT_POWERPOINT` | .pptx format |
| `QUICKTIME_MOVIE` | QuickTime video |
| `SLIDE_IMAGES` | Slide images |

**Video Resolutions:**
| Value | Description |
|-------|-------------|
| `f360p` | 360p video |
| `f540p` | 540p video |
| `f720p` | 720p video |
| `f1080p` | 1080p video |
| `f2160p` | 4K video |
| `NativeSize` | Native resolution |

### Codec

Video codec options.

| Value | Description |
|-------|-------------|
| `H264` | H.264 codec |
| `HEVC` | HEVC/H.265 codec |
| `APPLE_PRO_RES_422` | ProRes 422 |
| `APPLE_PRO_RES_422HQ` | ProRes 422 HQ |
| `APPLE_PRO_RES_422LT` | ProRes 422 LT |
| `APPLE_PRO_RES_422Proxy` | ProRes 422 Proxy |
| `APPLE_PRO_RES_4444` | ProRes 4444 |

### Framerate

Video framerate options.

| Value | FPS |
|-------|-----|
| `FPS_12` | 12 |
| `FPS_2398` | 23.98 |
| `FPS_24` | 24 |
| `FPS_25` | 25 |
| `FPS_2997` | 29.97 |
| `FPS_30` | 30 |
| `FPS_50` | 50 |
| `FPS_5994` | 59.94 |
| `FPS_60` | 60 |

### ObjectType

Creatable object types.

| Value | Description |
|-------|-------------|
| `DOCUMENT` | Document |
| `SLIDE` | Slide |
| `CHART` | Chart |
| `IMAGE` | Image |
| `SHAPE` | Shape |
| `LINE` | Line |
| `TABLE` | Table |
| `TEXT_ITEM` | Text item |
| `AUDIO_CLIP` | Audio clip |
| `MOVIE` | Movie |
| `GROUP` | Group |
| `TRANSITION_SETTINGS` | Transition settings |
| `iWORK_ITEM` | Generic iWork item |

### Transition

Slide transition effects.

| Value | Effect |
|-------|--------|
| `NONE` | No transition |
| `MAGIC_MOVE` | Magic Move (animate between slides) |
| `DISSOLVE` | Dissolve |
| `FADE_THROUGH_COLOR` | Fade through color |
| `FADE_AND_MOVE` | Fade and move |
| `PUSH` | Push |
| `REVEAL` | Reveal |
| `WIPE` | Wipe |
| `FLIP` | Flip |
| `CUBE` | Cube |
| `IRIS` | Iris |
| `BLINDS` | Blinds |
| `MOSAIC` | Mosaic |
| `GRID` | Grid |
| `CONFETTI` | Confetti |
| `SPARKLE` | Sparkle |
| `DROP` | Drop |
| `DROPLET` | Droplet |
| `FALL` | Fall |
| `SWING` | Swing |
| `TWIRL` | Twirl |
| `TWIST` | Twist |
| `SWAP` | Swap |
| `SWOOSH` | Swoosh |
| `SCALE` | Scale |
| `PERSPECTIVE` | Perspective |
| `PAGE_FLIP` | Page flip |
| `PIVOT` | Pivot |
| `DOORWAY` | Doorway |
| `REVOLVING_DOOR` | Revolving door |
| `FLOP` | Flop |
| `REFLECTION` | Reflection |
| `SWITCH` | Switch |
| `CLOTHESLINE` | Clothesline |
| `COLOR_PANES` | Color panes |
| `SHIMMER` | Shimmer |
| `MOVE_IN` | Move in |
| `OBJECT_CUBE` | Object cube |
| `OBJECT_FLIP` | Object flip |
| `OBJECT_POP` | Object pop |
| `OBJECT_PUSH` | Object push |
| `OBJECT_REVOLVE` | Object revolve |
| `OBJECT_ZOOM` | Object zoom |

### PrintSetting

Print options.

| Value | Description |
|-------|-------------|
| `INDIVIDUAL_SLIDES` | Print each slide |
| `SLIDE_WITH_NOTES` | Print slides with notes |
| `HANDOUTS` | Print as handouts |
| `STANDARD_ERROR_HANDLING` | Standard error handling |
| `DETAILED_ERROR_HANDLING` | Detailed error handling |

---

## Quick Reference Tables

### Common Operations

| Task | Code |
|------|------|
| Get Keynote app | `keynote = PyXA.Application("Keynote")` |
| Create document | `doc = keynote.new_document(theme=keynote.themes().by_name("White"))` |
| Get first document | `doc = keynote.documents()[0]` |
| Get all slides | `slides = doc.slides()` |
| Get slide by number | `slide = doc.slides().by_slide_number(3)` |
| Add slide | `slide = doc.new_slide({})` |
| Set slide title | `slide.default_title_item.objectText = "Title"` |
| Export to PDF | `doc.export("/path.pdf", ExportFormat.PDF)` |
| Export to PPTX | `doc.export("/path.pptx", ExportFormat.MICROSOFT_POWERPOINT)` |
| Start slideshow | `doc.start_from(doc.slides()[0])` |
| Stop slideshow | `doc.stop()` |

### Property Access Patterns

```python
# Single object
slide = doc.slides()[0]
print(slide.slide_number)

# Bulk access on lists
slides = doc.slides()
print(slides.slide_number())  # Returns list[int]
print(slides.skipped())       # Returns list[bool]

# Filtering
visible_slides = slides.by_skipped(False)
```

---

## See Also

- [PyXA Keynote Documentation](https://skaplanofficial.github.io/PyXA/reference/apps/keynote.html) - Official PyXA documentation
- [keynote-pyxa.md](keynote-pyxa.md) - Practical usage examples
- [keynote-basics.md](keynote-basics.md) - JXA fundamentals
- [keynote-recipes.md](keynote-recipes.md) - Common automation patterns
