# PyXA Core API Reference

Complete API reference for PyXA core modules: base classes, protocols, types, and error handling.

**Version**: 0.3.0 | **Python**: 3.10+ | **Source**: [PyXA GitHub](https://github.com/SKaplanOfficial/PyXA)

## Contents

- [Module Overview](#module-overview)
- [Class Hierarchy](#class-hierarchy)
- [XABase Module](#xabase-module)
  - [XAObject](#xaobject)
  - [XAList](#xalist)
  - [XAPath](#xapath)
  - [XAColor](#xacolor)
  - [XAURL](#xaurl)
  - [XAText](#xatext)
  - [XAImage](#xaimage)
  - [XASound](#xasound)
  - [XALocation](#xalocation)
  - [XAClipboard](#xaclipboard)
- [XABaseScriptable Module](#xabasescriptable-module)
  - [XASBApplication](#xasbapplication)
  - [XASBWindow](#xasbwindow)
  - [XASBWindowList](#xasbwindowlist)
  - [XASBPrintable](#xasbprintable)
- [XATypes Module](#xatypes-module)
- [XAProtocols Module](#xaprotocols-module)
- [XAErrors Module](#xaerrors-module)
- [Quick Reference Tables](#quick-reference-tables)
- [See Also](#see-also)

---

## Module Overview

| Module | Purpose |
|--------|---------|
| `PyXA.XABase` | Core wrapper classes for macOS objects (files, colors, URLs, text, images) |
| `PyXA.XABaseScriptable` | Base classes for scriptable applications and windows |
| `PyXA.XATypes` | Named tuple types for geometric data (points, rectangles) |
| `PyXA.XAProtocols` | Protocol definitions for capability interfaces |
| `PyXA.XAErrors` | Custom exception types for PyXA operations |

---

## Class Hierarchy

```
XAObject
├── XAPath
├── XAColor
├── XAURL
├── XAText
├── XAImage
├── XASound
├── XALocation
└── XASBObject
    ├── XASBApplication
    │   └── [App-specific: XASafariApplication, XAMailApplication, etc.]
    ├── XASBWindow
    └── XASBPrintable

XAList
├── XAPathList
├── XAURLList
├── XATextList
├── XAImageList
└── XASBWindowList

XAClipboard (standalone utility class)
```

---

## XABase Module

### XAObject

Base class for all PyXA wrapper objects. Provides common functionality for macOS object manipulation.

```python
class XAObject:
    """Base wrapper for macOS Scripting Bridge objects."""

    # Initialization
    def __init__(self, properties: dict = None) -> None

    # Properties
    @property
    def xa_elem(self) -> Any  # Underlying SBObject

    @property
    def xa_prnt(self) -> 'XAObject'  # Parent object reference

    # Methods
    def set_property(self, property_name: str, value: Any) -> None
    def has_element_with_properties(self, properties: dict) -> bool
```

**Example:**
```python
import PyXA

# All PyXA objects inherit from XAObject
safari = PyXA.Safari()  # XASafariApplication extends XASBApplication extends XAObject

# Access underlying Scripting Bridge object
sb_object = safari.xa_elem
```

---

### XAList

Base class for PyXA list objects. Provides bulk operations and fast enumeration patterns.

```python
class XAList(XAObject):
    """Wrapper for lists of scriptable objects with bulk operations."""

    # Initialization
    def __init__(self, properties: dict = None, filter: dict = None) -> None

    # Properties
    @property
    def xa_ocls(self) -> type  # Object class for list elements

    # List Operations
    def __len__(self) -> int
    def __iter__(self) -> Iterator
    def __getitem__(self, key: Union[int, slice]) -> Union[XAObject, 'XAList']

    # Filtering
    def by_property(self, property_name: str, value: Any) -> 'XAList'
    def by_name(self, name: str) -> XAObject
    def containing(self, property_name: str, value: str) -> 'XAList'

    # Bulk Property Access (Fast Enumeration)
    def name(self) -> list[str]
    def id(self) -> list[Any]

    # Transformations
    def filter(self, predicate: Callable) -> 'XAList'
    def first(self) -> XAObject
    def last(self) -> XAObject
    def at(self, index: int) -> XAObject
    def push(self, element: Union[XAObject, dict]) -> XAObject
```

**Example:**
```python
import PyXA

safari = PyXA.Safari()
tabs = safari.windows()[0].tabs()  # XAList of tabs

# Bulk property access (fast)
all_titles = tabs.name()  # Returns list of all tab names at once

# Filtering
github_tabs = tabs.containing("url", "github.com")

# Iteration
for tab in tabs:
    print(tab.url())
```

**★ Insight ─────────────────────────────────────**
XAList's bulk property access methods (like `.name()`, `.url()`) are dramatically faster than iterating and calling properties individually. PyXA uses Scripting Bridge's fast enumeration under the hood, making a single Apple Event call for all values.
**─────────────────────────────────────────────────**

---

### XAPath

Wrapper for file system paths with extensive file/folder operations.

```python
class XAPath(XAObject):
    """Wrapper for file system paths."""

    # Initialization
    def __init__(self, path: Union[str, 'XAPath', NSURL]) -> None

    # Properties
    @property
    def path(self) -> str  # POSIX path string

    @property
    def url(self) -> NSURL  # NSURL representation

    @property
    def name(self) -> str  # File/folder name

    @property
    def extension(self) -> str  # File extension

    @property
    def parent(self) -> 'XAPath'  # Parent directory

    @property
    def exists(self) -> bool  # Path exists on disk

    @property
    def is_file(self) -> bool

    @property
    def is_directory(self) -> bool

    # File Operations
    def open(self) -> None  # Open with default app
    def reveal_in_finder(self) -> None
    def move_to(self, destination: 'XAPath') -> 'XAPath'
    def copy_to(self, destination: 'XAPath') -> 'XAPath'
    def delete(self) -> None  # Move to Trash

    # Content Operations
    def read_text(self, encoding: str = 'utf-8') -> str
    def write_text(self, content: str, encoding: str = 'utf-8') -> None
    def read_data(self) -> bytes
    def write_data(self, data: bytes) -> None
```

**Example:**
```python
import PyXA

# Create path object
doc = PyXA.XAPath("~/Documents/report.txt")

# Check properties
if doc.exists:
    content = doc.read_text()
    print(f"File: {doc.name}, Extension: {doc.extension}")

# Reveal in Finder
doc.reveal_in_finder()

# Parent directory
parent = doc.parent  # ~/Documents as XAPath
```

---

### XAColor

Wrapper for color values with conversion between color spaces.

```python
class XAColor(XAObject):
    """Wrapper for NSColor with color space conversions."""

    # Class Methods (Preset Colors)
    @classmethod
    def white(cls) -> 'XAColor'
    @classmethod
    def black(cls) -> 'XAColor'
    @classmethod
    def red(cls) -> 'XAColor'
    @classmethod
    def green(cls) -> 'XAColor'
    @classmethod
    def blue(cls) -> 'XAColor'
    @classmethod
    def yellow(cls) -> 'XAColor'
    @classmethod
    def orange(cls) -> 'XAColor'
    @classmethod
    def purple(cls) -> 'XAColor'
    @classmethod
    def gray(cls) -> 'XAColor'
    @classmethod
    def clear(cls) -> 'XAColor'  # Transparent

    # Initialization
    @classmethod
    def from_hex(cls, hex_string: str) -> 'XAColor'
    @classmethod
    def from_rgb(cls, red: float, green: float, blue: float, alpha: float = 1.0) -> 'XAColor'
    @classmethod
    def from_hsb(cls, hue: float, saturation: float, brightness: float, alpha: float = 1.0) -> 'XAColor'

    # Properties
    @property
    def hex_value(self) -> str  # "#RRGGBB" format

    @property
    def red_value(self) -> float  # 0.0-1.0
    @property
    def green_value(self) -> float
    @property
    def blue_value(self) -> float
    @property
    def alpha_value(self) -> float

    @property
    def hue_value(self) -> float  # 0.0-1.0
    @property
    def saturation_value(self) -> float
    @property
    def brightness_value(self) -> float

    # Methods
    def make_swatch(self, width: int = 100, height: int = 100) -> 'XAImage'
    def mix_with(self, color: 'XAColor', fraction: float = 0.5) -> 'XAColor'
```

**Example:**
```python
import PyXA

# Preset colors
bg_color = PyXA.XAColor.blue()

# From hex
accent = PyXA.XAColor.from_hex("#FF6B35")

# From RGB (values 0.0-1.0)
custom = PyXA.XAColor.from_rgb(0.5, 0.7, 0.9, 1.0)

# Convert to hex
print(f"Hex: {custom.hex_value}")  # "#7FB2E5"

# Mix colors
blended = bg_color.mix_with(accent, 0.3)
```

---

### XAURL

Wrapper for URL handling with web and file URL support.

```python
class XAURL(XAObject):
    """Wrapper for NSURL with URL parsing and manipulation."""

    # Initialization
    def __init__(self, url: Union[str, NSURL]) -> None

    # Properties
    @property
    def url(self) -> str  # Full URL string

    @property
    def scheme(self) -> str  # http, https, file, etc.

    @property
    def host(self) -> str  # Domain name

    @property
    def port(self) -> int

    @property
    def path(self) -> str  # Path component

    @property
    def query(self) -> str  # Query string

    @property
    def fragment(self) -> str  # Fragment identifier

    # Methods
    def open(self) -> None  # Open in default browser/app
    def open_with(self, app: 'XASBApplication') -> None
```

**Example:**
```python
import PyXA

url = PyXA.XAURL("https://github.com/SKaplanOfficial/PyXA?tab=readme")

print(f"Host: {url.host}")      # github.com
print(f"Path: {url.path}")      # /SKaplanOfficial/PyXA
print(f"Query: {url.query}")    # tab=readme

# Open in browser
url.open()
```

---

### XAText

Rich text wrapper with formatting and manipulation capabilities.

```python
class XAText(XAObject):
    """Wrapper for NSAttributedString with rich text operations."""

    # Initialization
    def __init__(self, text: Union[str, NSAttributedString] = "") -> None

    # Properties
    @property
    def text(self) -> str  # Plain text content

    @property
    def length(self) -> int

    # Text Manipulation
    def words(self) -> list[str]
    def sentences(self) -> list[str]
    def paragraphs(self) -> list[str]

    def replace(self, old: str, new: str) -> 'XAText'
    def uppercase(self) -> 'XAText'
    def lowercase(self) -> 'XAText'
    def capitalize(self) -> 'XAText'

    # Rich Text Operations
    def set_font(self, font_name: str, size: float = 12.0) -> 'XAText'
    def set_color(self, color: XAColor) -> 'XAText'
    def bold(self) -> 'XAText'
    def italic(self) -> 'XAText'
    def underline(self) -> 'XAText'

    # Clipboard
    def copy_to_clipboard(self) -> None
```

**Example:**
```python
import PyXA

text = PyXA.XAText("Hello World! This is PyXA.")

# Analysis
words = text.words()        # ['Hello', 'World!', 'This', 'is', 'PyXA.']
sentences = text.sentences() # ['Hello World!', 'This is PyXA.']

# Transformation
upper = text.uppercase()  # "HELLO WORLD! THIS IS PYXA."

# Styling
styled = text.set_font("Helvetica", 18).bold().set_color(PyXA.XAColor.blue())
styled.copy_to_clipboard()
```

---

### XAImage

Image wrapper with processing and export capabilities.

```python
class XAImage(XAObject):
    """Wrapper for NSImage with image processing."""

    # Initialization
    def __init__(self, image: Union[str, 'XAPath', NSImage, bytes]) -> None

    @classmethod
    def from_clipboard(cls) -> 'XAImage'

    @classmethod
    def from_url(cls, url: Union[str, XAURL]) -> 'XAImage'

    @classmethod
    def from_screen(cls, region: tuple = None) -> 'XAImage'  # Screenshot

    # Properties
    @property
    def size(self) -> tuple[int, int]  # (width, height)

    @property
    def width(self) -> int

    @property
    def height(self) -> int

    # Transformations
    def resize(self, width: int, height: int) -> 'XAImage'
    def scale(self, factor: float) -> 'XAImage'
    def crop(self, x: int, y: int, width: int, height: int) -> 'XAImage'
    def rotate(self, degrees: float) -> 'XAImage'
    def flip_horizontal(self) -> 'XAImage'
    def flip_vertical(self) -> 'XAImage'

    # Filters
    def grayscale(self) -> 'XAImage'
    def sepia(self) -> 'XAImage'
    def invert(self) -> 'XAImage'
    def blur(self, radius: float = 10) -> 'XAImage'
    def sharpen(self) -> 'XAImage'

    # Export
    def save(self, path: Union[str, XAPath], format: str = 'png') -> XAPath
    def copy_to_clipboard(self) -> None
    def show_in_preview(self) -> None

    # OCR (Vision framework)
    def extract_text(self) -> str
```

**Example:**
```python
import PyXA

# Load image
img = PyXA.XAImage("~/Pictures/photo.png")

# Transform
processed = (img
    .resize(800, 600)
    .grayscale()
    .blur(5)
)

# Save
processed.save("~/Desktop/processed.png")

# Screenshot
screenshot = PyXA.XAImage.from_screen()
text = screenshot.extract_text()  # OCR
```

---

### XASound

Audio playback and manipulation wrapper.

```python
class XASound(XAObject):
    """Wrapper for NSSound with audio playback."""

    # Initialization
    def __init__(self, sound: Union[str, XAPath, NSSound]) -> None

    @classmethod
    def from_name(cls, name: str) -> 'XASound'  # System sounds

    # Properties
    @property
    def name(self) -> str

    @property
    def duration(self) -> float  # Seconds

    @property
    def volume(self) -> float  # 0.0-1.0
    @volume.setter
    def volume(self, value: float) -> None

    @property
    def is_playing(self) -> bool

    # Playback
    def play(self) -> None
    def pause(self) -> None
    def stop(self) -> None
    def resume(self) -> None
```

**Example:**
```python
import PyXA

# System sound
beep = PyXA.XASound.from_name("Ping")
beep.play()

# Custom sound
music = PyXA.XASound("~/Music/track.mp3")
music.volume = 0.5
music.play()
```

---

### XALocation

Geolocation wrapper for coordinate handling.

```python
class XALocation(XAObject):
    """Wrapper for CLLocation with coordinate operations."""

    # Initialization
    def __init__(self, latitude: float, longitude: float) -> None

    @classmethod
    def current_location(cls) -> 'XALocation'  # Requires location permissions

    # Properties
    @property
    def latitude(self) -> float

    @property
    def longitude(self) -> float

    @property
    def coordinates(self) -> tuple[float, float]

    # Methods
    def distance_to(self, other: 'XALocation') -> float  # Meters
    def reverse_geocode(self) -> dict  # Address components
    def open_in_maps(self) -> None
```

**Example:**
```python
import PyXA

# Create location
office = PyXA.XALocation(37.7749, -122.4194)  # San Francisco

# Current location (requires permission)
current = PyXA.XALocation.current_location()

# Calculate distance
distance = current.distance_to(office)
print(f"Distance: {distance / 1000:.1f} km")

# Reverse geocode
address = office.reverse_geocode()
print(f"City: {address.get('city')}")
```

---

### XAClipboard

System clipboard access utility class.

```python
class XAClipboard:
    """Utility class for system clipboard operations."""

    # Class Methods
    @classmethod
    def get_text(cls) -> str

    @classmethod
    def set_text(cls, text: str) -> None

    @classmethod
    def get_image(cls) -> XAImage

    @classmethod
    def set_image(cls, image: XAImage) -> None

    @classmethod
    def get_urls(cls) -> list[XAURL]

    @classmethod
    def set_urls(cls, urls: list[XAURL]) -> None

    @classmethod
    def get_files(cls) -> list[XAPath]

    @classmethod
    def set_files(cls, files: list[XAPath]) -> None

    @classmethod
    def clear(cls) -> None

    @classmethod
    def content_types(cls) -> list[str]  # Available types on clipboard
```

**Example:**
```python
import PyXA

# Text operations
PyXA.XAClipboard.set_text("Hello from PyXA!")
text = PyXA.XAClipboard.get_text()

# Image operations
screenshot = PyXA.XAImage.from_screen()
PyXA.XAClipboard.set_image(screenshot)

# Check available types
types = PyXA.XAClipboard.content_types()
if 'public.url' in types:
    urls = PyXA.XAClipboard.get_urls()
```

---

## XABaseScriptable Module

### XASBApplication

Base class for scriptable application wrappers.

```python
class XASBApplication(XAObject):
    """Base class for scriptable applications."""

    # Initialization
    def __init__(self, properties: dict = None) -> None

    # Properties
    @property
    def name(self) -> str  # Application name

    @property
    def bundle_identifier(self) -> str  # e.g., "com.apple.Safari"

    @property
    def version(self) -> str

    @property
    def frontmost(self) -> bool  # Is active application

    @property
    def visible(self) -> bool
    @visible.setter
    def visible(self, value: bool) -> None

    # State
    def running(self) -> bool

    # Actions
    def activate(self) -> None  # Bring to front
    def hide(self) -> None
    def quit(self, saving: str = 'yes') -> None  # 'yes', 'no', 'ask'

    # Windows
    def windows(self, filter: dict = None) -> XASBWindowList
    def front_window(self) -> XASBWindow

    # Menu bar (UI scripting)
    def menu_bar(self, index: int = 0) -> 'XAUIElement'
```

**Example:**
```python
import PyXA

# Get Safari application
safari = PyXA.Safari()  # Inherits from XASBApplication

# Check state
if not safari.running():
    safari.activate()

# Bring to front
safari.activate()

# Access windows
windows = safari.windows()
front = safari.front_window()
```

---

### XASBWindow

Base class for application window wrappers.

```python
class XASBWindow(XAObject):
    """Base class for application windows."""

    # Properties
    @property
    def name(self) -> str  # Window title

    @property
    def id(self) -> int  # Unique window ID

    @property
    def index(self) -> int  # Window stack order
    @index.setter
    def index(self, value: int) -> None

    @property
    def bounds(self) -> tuple[int, int, int, int]  # (x, y, width, height)
    @bounds.setter
    def bounds(self, value: tuple) -> None

    @property
    def position(self) -> tuple[int, int]  # (x, y)
    @position.setter
    def position(self, value: tuple) -> None

    @property
    def size(self) -> tuple[int, int]  # (width, height)
    @size.setter
    def size(self, value: tuple) -> None

    @property
    def visible(self) -> bool
    @visible.setter
    def visible(self, value: bool) -> None

    @property
    def miniaturized(self) -> bool  # Minimized to dock
    @miniaturized.setter
    def miniaturized(self, value: bool) -> None

    @property
    def zoomed(self) -> bool  # Maximized
    @zoomed.setter
    def zoomed(self, value: bool) -> None

    # Actions
    def close(self) -> None
```

**Example:**
```python
import PyXA

safari = PyXA.Safari()
window = safari.front_window()

# Get window info
print(f"Title: {window.name()}")
print(f"Position: {window.position}")
print(f"Size: {window.size}")

# Manipulate window
window.position = (100, 100)
window.size = (1200, 800)
window.zoomed = True  # Maximize
```

---

### XASBWindowList

List wrapper for application windows with bulk operations.

```python
class XASBWindowList(XAList):
    """List of application windows."""

    # Bulk Property Access
    def name(self) -> list[str]
    def id(self) -> list[int]
    def index(self) -> list[int]
    def bounds(self) -> list[tuple]
    def visible(self) -> list[bool]
    def miniaturized(self) -> list[bool]

    # Filtering
    def by_name(self, name: str) -> XASBWindow
    def by_id(self, id: int) -> XASBWindow

    # Actions
    def close(self) -> None  # Close all windows
```

---

### XASBPrintable

Mixin protocol for objects that support printing.

```python
class XASBPrintable:
    """Mixin for printable objects."""

    def print(self,
              print_dialog: bool = True,
              copies: int = 1,
              collating: bool = True,
              pages_across: int = 1,
              pages_down: int = 1) -> None
```

---

## XATypes Module

Named tuple types for geometric and temporal data.

```python
from typing import NamedTuple

class XAPoint(NamedTuple):
    """2D coordinate point."""
    x: float
    y: float

class XARectangle(NamedTuple):
    """Rectangle defined by origin and size."""
    x: float
    y: float
    width: float
    height: float

class XADatetimeBlock(NamedTuple):
    """Date/time range for scheduling."""
    start_date: datetime
    end_date: datetime
```

**Example:**
```python
from PyXA import XAPoint, XARectangle, XADatetimeBlock
from datetime import datetime, timedelta

# Point
position = XAPoint(100, 200)
print(f"X: {position.x}, Y: {position.y}")

# Rectangle
bounds = XARectangle(0, 0, 1920, 1080)
area = bounds.width * bounds.height

# Date block
meeting = XADatetimeBlock(
    start_date=datetime.now(),
    end_date=datetime.now() + timedelta(hours=1)
)
```

---

## XAProtocols Module

Protocol interfaces defining object capabilities.

### Protocol Summary

| Protocol | Purpose | Key Methods |
|----------|---------|-------------|
| `XAProtocol` | Base marker protocol | - |
| `XACanOpenPath` | Can open file paths | `open(path)` |
| `XACanPrintPath` | Can print file paths | `print(path)` |
| `XAClipboardCodable` | Clipboard copy/paste | `copy_to_clipboard()` |
| `XACloseable` | Can be closed | `close()` |
| `XADeletable` | Can be deleted | `delete()` |
| `XAImageLike` | Image conversion | `to_image() -> XAImage` |
| `XAPathLike` | Path conversion | `to_path() -> XAPath` |
| `XAPrintable` | Printing support | `print()` |
| `XASelectable` | Selection support | `select()` |
| `XAShowable` | Display/reveal | `show()` |

### Protocol Definitions

```python
from typing import Protocol

class XAClipboardCodable(Protocol):
    """Protocol for clipboard-compatible objects."""
    def copy_to_clipboard(self) -> None: ...

class XACloseable(Protocol):
    """Protocol for closeable objects."""
    def close(self, saving: str = 'yes') -> None: ...

class XADeletable(Protocol):
    """Protocol for deletable objects."""
    def delete(self) -> None: ...

class XAImageLike(Protocol):
    """Protocol for objects convertible to images."""
    def to_image(self) -> 'XAImage': ...

class XAPathLike(Protocol):
    """Protocol for objects with file paths."""
    def to_path(self) -> 'XAPath': ...
    @property
    def path(self) -> str: ...

class XAPrintable(Protocol):
    """Protocol for printable objects."""
    def print(self, **options) -> None: ...

class XASelectable(Protocol):
    """Protocol for selectable objects."""
    def select(self) -> None: ...

class XAShowable(Protocol):
    """Protocol for objects that can be revealed/shown."""
    def show(self) -> None: ...
```

---

## XAErrors Module

Custom exception types for PyXA operations.

### Exception Hierarchy

```
Exception
└── PyXAError (base)
    ├── AppleScriptError
    ├── ApplicationNotFoundError
    ├── AuthenticationError
    ├── InvalidPredicateError
    └── UnconstructableClassError
```

### Exception Definitions

```python
class AppleScriptError(Exception):
    """Error executing AppleScript code.

    Attributes:
        script: The AppleScript that failed
        error_message: Description from AppleScript engine
        error_number: AppleScript error code
    """
    def __init__(self, script: str, error_message: str, error_number: int = -1):
        self.script = script
        self.error_message = error_message
        self.error_number = error_number

class ApplicationNotFoundError(Exception):
    """Application not installed or cannot be found.

    Attributes:
        app_name: Name of the application
        bundle_id: Bundle identifier if provided
    """
    def __init__(self, app_name: str, bundle_id: str = None):
        self.app_name = app_name
        self.bundle_id = bundle_id

class AuthenticationError(Exception):
    """Permission denied or authentication required.

    Attributes:
        app_name: Application requiring permission
        permission_type: Type of permission needed (Automation, Accessibility, etc.)
    """
    def __init__(self, app_name: str, permission_type: str):
        self.app_name = app_name
        self.permission_type = permission_type

class InvalidPredicateError(Exception):
    """Invalid filter predicate or query.

    Attributes:
        predicate: The invalid predicate string
        reason: Why it's invalid
    """
    def __init__(self, predicate: str, reason: str):
        self.predicate = predicate
        self.reason = reason

class UnconstructableClassError(Exception):
    """Cannot instantiate the requested class.

    Attributes:
        class_name: Name of the class
        reason: Why construction failed
    """
    def __init__(self, class_name: str, reason: str):
        self.class_name = class_name
        self.reason = reason
```

### Error Handling Example

```python
import PyXA
from PyXA.XAErrors import (
    ApplicationNotFoundError,
    AuthenticationError,
    AppleScriptError
)

try:
    # Attempt to automate an app
    mail = PyXA.Mail()
    inbox = mail.inboxes()[0]
    messages = inbox.messages()

except ApplicationNotFoundError as e:
    print(f"App not found: {e.app_name}")
    print("Please install the application and try again.")

except AuthenticationError as e:
    print(f"Permission denied for {e.app_name}")
    print(f"Grant '{e.permission_type}' permission in System Settings")
    print("System Settings > Privacy & Security > Automation")

except AppleScriptError as e:
    print(f"Script error ({e.error_number}): {e.error_message}")
    print(f"Failed script: {e.script[:100]}...")
```

---

## Quick Reference Tables

### Common Object Patterns

| Task | Code |
|------|------|
| Get application | `app = PyXA.Safari()` |
| Check if running | `app.running()` |
| Activate app | `app.activate()` |
| Get front window | `app.front_window()` |
| List all windows | `app.windows()` |
| Get window by name | `app.windows().by_name("Title")` |

### Clipboard Operations

| Task | Code |
|------|------|
| Get text | `PyXA.XAClipboard.get_text()` |
| Set text | `PyXA.XAClipboard.set_text("Hello")` |
| Get image | `PyXA.XAClipboard.get_image()` |
| Set image | `PyXA.XAClipboard.set_image(img)` |
| Get files | `PyXA.XAClipboard.get_files()` |
| Clear | `PyXA.XAClipboard.clear()` |

### Color Operations

| Task | Code |
|------|------|
| Preset color | `PyXA.XAColor.blue()` |
| From hex | `PyXA.XAColor.from_hex("#FF6B35")` |
| From RGB | `PyXA.XAColor.from_rgb(1.0, 0.5, 0.2)` |
| To hex | `color.hex_value` |
| Mix colors | `color1.mix_with(color2, 0.5)` |

### File Operations

| Task | Code |
|------|------|
| Create path | `PyXA.XAPath("~/Documents/file.txt")` |
| Check exists | `path.exists` |
| Read text | `path.read_text()` |
| Write text | `path.write_text("content")` |
| Open file | `path.open()` |
| Reveal in Finder | `path.reveal_in_finder()` |

### Image Operations

| Task | Code |
|------|------|
| Load image | `PyXA.XAImage("path/to/image.png")` |
| Screenshot | `PyXA.XAImage.from_screen()` |
| From clipboard | `PyXA.XAImage.from_clipboard()` |
| Resize | `img.resize(800, 600)` |
| Grayscale | `img.grayscale()` |
| Save | `img.save("output.png")` |
| OCR | `img.extract_text()` |

---

## See Also

- **Official PyXA Documentation**: https://skaplanofficial.github.io/PyXA/
- **PyXA GitHub**: https://github.com/SKaplanOfficial/PyXA
- **PyXA Basics Tutorial**: `automating-mac-apps/references/pyxa-basics.md`
- **AppleScript to PyXA Migration**: `automating-mac-apps/references/applescript-to-pyxa-conversion.md`
- **App-Specific PyXA References**:
  - Calendar: `automating-calendar/references/calendar-pyxa-api-reference.md`
  - Contacts: `automating-contacts/references/contacts-pyxa-api-reference.md`
  - Keynote: `automating-keynote/references/keynote-pyxa-api-reference.md`
  - Mail: `automating-mail/references/mail-pyxa-api-reference.md`
  - Notes: `automating-notes/references/notes-pyxa-api-reference.md`
  - Numbers: `automating-numbers/references/numbers-pyxa-api-reference.md`
  - Reminders: `automating-reminders/references/reminders-pyxa-api-reference.md`
